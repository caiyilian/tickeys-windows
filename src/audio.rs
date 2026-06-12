use std::collections::VecDeque;
use std::ffi::CString;
use std::path::PathBuf;
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static mut DEVICE: *mut std::ffi::c_void = ptr::null_mut();
static mut CONTEXT: *mut std::ffi::c_void = ptr::null_mut();

pub fn get_resource_path(sub: &str) -> PathBuf {
    let candidates: [&dyn Fn() -> Option<PathBuf>; 4] = [
        &|| {
            let exe = std::env::current_exe().ok()?;
            let dir = exe.parent()?.join("data").join(sub);
            if dir.exists() { Some(dir) } else { None }
        },
        &|| {
            let exe = std::env::current_exe().ok()?;
            let dir = exe.parent()?.parent()?.parent()?.join("resource").join("data").join(sub);
            if dir.exists() { Some(dir) } else { None }
        },
        &|| {
            let dir = PathBuf::from("resource/data").join(sub);
            if dir.exists() { Some(dir) } else { None }
        },
        &|| {
            let dir = PathBuf::from("data").join(sub);
            if dir.exists() { Some(dir) } else { None }
        },
    ];
    for f in &candidates {
        if let Some(p) = f() {
            return p;
        }
    }
    PathBuf::from("resource/data").join(sub)
}

pub struct AudioData {
    buffer: ALuint,
}

impl AudioData {
    pub fn from_file(path: &Path) -> Result<AudioData, String> {
        let mut reader = hound::WavReader::open(path)
            .map_err(|e| format!("Failed to open WAV file {:?}: {}", path, e))?;
        let spec = reader.spec();

        let format = match (spec.channels, spec.bits_per_sample) {
            (1, 16) => AL_FORMAT_MONO16,
            (2, 16) => AL_FORMAT_STEREO16,
            (1, 8) => AL_FORMAT_MONO8,
            (2, 8) => AL_FORMAT_STEREO8,
            _ => return Err(format!("Unsupported WAV format: {}ch {}bit", spec.channels, spec.bits_per_sample)),
        };

        let samples: Vec<u8> = if spec.bits_per_sample == 16 {
            reader.samples::<i16>()
                .filter_map(Result::ok)
                .flat_map(|s| s.to_ne_bytes())
                .collect()
        } else {
            reader.samples::<i8>()
                .filter_map(Result::ok)
                .map(|s| s as u8)
                .collect()
        };

        let mut buffer: ALuint = 0;
        unsafe {
            alGenBuffers(1, &mut buffer);
            alBufferData(buffer, format, samples.as_ptr() as *const std::ffi::c_void, samples.len() as ALsizei, spec.sample_rate as ALsizei);
            let err = alGetError();
            if err != AL_NO_ERROR {
                alDeleteBuffers(1, &buffer);
                return Err(format!("OpenAL alBufferData error: 0x{:X}", err));
            }
        }

        log::info!("Loaded WAV: {:?} ({}ch {}bit {}hz)", path, spec.channels, spec.bits_per_sample, spec.sample_rate);
        Ok(AudioData { buffer })
    }

    pub fn id(&self) -> ALuint {
        self.buffer
    }
}

impl Drop for AudioData {
    fn drop(&mut self) {
        unsafe {
            alDeleteBuffers(1, &self.buffer);
        }
    }
}

pub struct AudioSource {
    id: ALuint,
}

impl AudioSource {
    pub fn new() -> Option<AudioSource> {
        let mut id: ALuint = 0;
        unsafe {
            alGenSources(1, &mut id);
        }
        match unsafe { alGetError() } {
            AL_NO_ERROR => Some(AudioSource { id }),
            _ => None,
        }
    }

    pub fn connect_to_buffer(&mut self, data: &AudioData) {
        self.stop();
        unsafe {
            alSourcei(self.id, AL_BUFFER, data.id() as ALint);
        }
    }

    pub fn disconnect_from_buffer(&mut self) {
        unsafe {
            alSourceStop(self.id);
            alSourcei(self.id, AL_BUFFER, 0);
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        unsafe { alSourcef(self.id, AL_GAIN, gain); }
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        unsafe { alSourcef(self.id, AL_PITCH, pitch); }
    }

    pub fn play(&mut self) {
        unsafe { alSourcePlay(self.id); }
    }

    pub fn stop(&mut self) {
        unsafe { alSourceStop(self.id); }
    }
}

impl Drop for AudioSource {
    fn drop(&mut self) {
        self.stop();
        unsafe { alDeleteSources(1, &self.id); }
    }
}

pub struct SimpleAudioPlayer {
    data: Vec<AudioData>,
    source_cache: VecDeque<AudioSource>,
    _max_source_count: usize,
}

impl SimpleAudioPlayer {
    pub fn new(max_source_count: usize) -> SimpleAudioPlayer {
        assert!(max_source_count > 0);
        let mut sources = VecDeque::with_capacity(max_source_count);
        for _ in 0..max_source_count {
            sources.push_back(AudioSource::new().unwrap());
        }
        SimpleAudioPlayer {
            data: Vec::new(),
            source_cache: sources,
            _max_source_count: max_source_count,
        }
    }

    pub fn load_data(&mut self, data: Vec<AudioData>) {
        for s in self.source_cache.iter_mut() {
            s.disconnect_from_buffer();
        }
        self.data = data;
    }

    pub fn set_gain(&mut self, gain: f32) {
        for s in self.source_cache.iter_mut() {
            s.set_gain(gain);
        }
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        for s in self.source_cache.iter_mut() {
            s.set_pitch(pitch);
        }
    }

    pub fn play(&mut self, index: usize) {
        let data = match self.data.get(index) {
            Some(val) => val,
            None => return,
        };
        let mut oldest_source = self.source_cache.pop_front().unwrap();
        oldest_source.connect_to_buffer(data);
        oldest_source.play();
        self.source_cache.push_back(oldest_source);
    }

    pub fn unload_data(&mut self) {
        self.source_cache.clear();
        self.data.clear();
    }

    pub fn rebuild(&mut self, max_source_count: usize) {
        if max_source_count == 0 || max_source_count == self._max_source_count {
            return;
        }
        let current_data = std::mem::take(&mut self.data);
        let mut sources = VecDeque::with_capacity(max_source_count);
        for _ in 0..max_source_count {
            sources.push_back(AudioSource::new().unwrap());
        }
        self.source_cache = sources;
        self._max_source_count = max_source_count;
        self.load_data(current_data);
        log::info!("Player rebuilt with {max_source_count} sources");
    }
}

impl Drop for SimpleAudioPlayer {
    fn drop(&mut self) {
        self.unload_data();
    }
}

static PLAYER: Mutex<Option<SimpleAudioPlayer>> = Mutex::new(None);
static MUTED: AtomicBool = AtomicBool::new(false);

pub fn set_mute(muted: bool) {
    MUTED.store(muted, Ordering::Relaxed);
    if muted {
        log::info!("Audio muted");
    } else {
        log::info!("Audio unmuted");
    }
}

pub fn is_muted() -> bool {
    MUTED.load(Ordering::Relaxed)
}

pub fn init_player(max_sources: usize) {
    let mut guard = PLAYER.lock().unwrap();
    *guard = Some(SimpleAudioPlayer::new(max_sources));
}

pub fn load_audio_data(data: Vec<AudioData>) {
    let mut guard = PLAYER.lock().unwrap();
    if let Some(ref mut player) = *guard {
        player.load_data(data);
    }
}

pub fn play_audio(index: usize) {
    if MUTED.load(Ordering::Relaxed) {
        return;
    }
    let mut guard = PLAYER.lock().unwrap();
    if let Some(ref mut player) = *guard {
        player.play(index);
    }
}

pub fn set_volume(volume: f32) {
    let mut guard = PLAYER.lock().unwrap();
    if let Some(ref mut player) = *guard {
        player.set_gain(volume);
    }
}

pub fn set_pitch(pitch: f32) {
    let mut guard = PLAYER.lock().unwrap();
    if let Some(ref mut player) = *guard {
        player.set_pitch(pitch);
    }
}

pub fn shutdown_player() {
    let mut guard = PLAYER.lock().unwrap();
    *guard = None;
}

pub fn player_is_initialized() -> bool {
    let guard = PLAYER.lock().unwrap();
    guard.is_some()
}

pub fn rebuild_player(max_sources: usize) {
    let mut guard = PLAYER.lock().unwrap();
    if let Some(ref mut player) = *guard {
        player.rebuild(max_sources);
    }
}

pub fn init() -> Result<(), String> {
    unsafe {
        let device_name = CString::new("OpenAL Soft").unwrap();
        DEVICE = alcOpenDevice(device_name.as_ptr());
        if DEVICE.is_null() {
            DEVICE = alcOpenDevice(ptr::null());
        }
        if DEVICE.is_null() {
            return Err("Failed to open OpenAL device".to_string());
        }

        CONTEXT = alcCreateContext(DEVICE, ptr::null());
        if CONTEXT.is_null() {
            alcCloseDevice(DEVICE);
            DEVICE = ptr::null_mut();
            return Err("Failed to create OpenAL context".to_string());
        }

        if alcMakeContextCurrent(CONTEXT) == ALC_FALSE {
            alcDestroyContext(CONTEXT);
            CONTEXT = ptr::null_mut();
            alcCloseDevice(DEVICE);
            DEVICE = ptr::null_mut();
            return Err("Failed to make OpenAL context current".to_string());
        }

        log::info!("OpenAL initialized successfully");
        Ok(())
    }
}

pub fn shutdown() {
    shutdown_player();
    unsafe {
        if !CONTEXT.is_null() {
            alcMakeContextCurrent(ptr::null_mut());
            alcDestroyContext(CONTEXT);
            CONTEXT = ptr::null_mut();
        }
        if !DEVICE.is_null() {
            alcCloseDevice(DEVICE);
            DEVICE = ptr::null_mut();
        }
        log::info!("OpenAL shutdown");
    }
}

type ALCboolean = i32;
type ALCdevice = *mut std::ffi::c_void;
type ALCcontext = *mut std::ffi::c_void;
type ALenum = i32;
type ALint = i32;
type ALuint = u32;
type ALsizei = i32;
type ALfloat = f32;

const ALC_FALSE: ALCboolean = 0;
const AL_NO_ERROR: ALenum = 0;
const AL_BUFFER: ALenum = 0x1009;
const AL_PITCH: ALenum = 0x1003;
const AL_GAIN: ALenum = 0x100A;
const AL_FORMAT_MONO8: ALenum = 0x1100;
const AL_FORMAT_MONO16: ALenum = 0x1101;
const AL_FORMAT_STEREO8: ALenum = 0x1102;
const AL_FORMAT_STEREO16: ALenum = 0x1103;

#[link(name = "OpenAL32")]
extern "system" {
    fn alcOpenDevice(deviceName: *const i8) -> ALCdevice;
    fn alcCreateContext(device: ALCdevice, attrList: *const i32) -> ALCcontext;
    fn alcMakeContextCurrent(context: ALCcontext) -> ALCboolean;
    fn alcDestroyContext(context: ALCcontext);
    fn alcCloseDevice(device: ALCdevice) -> ALCboolean;

    fn alGenBuffers(n: ALsizei, buffers: *mut ALuint);
    fn alBufferData(buffer: ALuint, format: ALenum, data: *const std::ffi::c_void, size: ALsizei, freq: ALsizei);
    fn alDeleteBuffers(n: ALsizei, buffers: *const ALuint);
    fn alGenSources(n: ALsizei, sources: *mut ALuint);
    fn alSourcei(source: ALuint, param: ALenum, value: ALint);
    fn alSourcef(source: ALuint, param: ALenum, value: ALfloat);
    fn alSourcePlay(source: ALuint);
    fn alSourceStop(source: ALuint);
    fn alDeleteSources(n: ALsizei, sources: *const ALuint);
    fn alGetError() -> ALenum;
}
