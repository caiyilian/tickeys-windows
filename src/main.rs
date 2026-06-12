#![allow(static_mut_refs)]

mod audio;
mod config;
#[allow(dead_code)]
mod consts;
#[allow(dead_code)]
mod filter;
#[allow(dead_code)]
mod gui;
mod keyboard;
mod logging;
#[allow(dead_code)]
mod power;
mod schemes;
mod tray;

use consts::*;
use std::collections::BTreeMap;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut KEYMAP: Option<BTreeMap<u8, u8>> = None;
static mut FIRST_N_NON_UNIQUE: i16 = -1;

fn switch_scheme(name: &str) {
    let scheme = match schemes::find_scheme(name) {
        Some(s) => s,
        None => {
            log::error!("Scheme '{}' not found", name);
            return;
        }
    };

    let scheme_dir = audio::get_resource_path(&scheme.name);
    let mut audio_data = Vec::with_capacity(scheme.files.len());
    for f in &scheme.files {
        let path = scheme_dir.join(f);
        match audio::AudioData::from_file(&path) {
            Ok(data) => audio_data.push(data),
            Err(e) => log::error!("{e}"),
        }
    }

    if audio_data.is_empty() {
        log::warn!("No audio files loaded for scheme '{}'", scheme.name);
        return;
    }

    unsafe {
        KEYMAP = Some(scheme.key_audio_map.clone());
        FIRST_N_NON_UNIQUE = scheme.non_unique_count as i16;
    }

    let player_exists = audio::player_is_initialized();
    if player_exists {
        audio::rebuild_player(audio_data.len().max(4));
        audio::load_audio_data(audio_data);
    } else {
        audio::init_player(audio_data.len().max(4));
        audio::load_audio_data(audio_data);
    }

    apply_audio_settings();

    log::info!("Switched to scheme: {} ({} sounds)", scheme.display_name, scheme.files.len());

    if let Some(mut cfg) = config::get_config() {
        cfg.scheme = scheme.name.clone();
        config::update_config(&cfg);
    }
}

fn apply_audio_settings() {
    if let Some(cfg) = config::get_config() {
        audio::set_volume(cfg.volume);
        audio::set_pitch(cfg.pitch);
    }
}

fn load_default_scheme() {
    schemes::load_schemes();
    let cfg = config::init_config();

    let scheme_name = if cfg.scheme.is_empty() || schemes::find_scheme(&cfg.scheme).is_none() {
        match schemes::first_scheme_name() {
            Some(name) => {
                let mut c = cfg.clone();
                c.scheme = name.clone();
                config::update_config(&c);
                name
            }
            None => {
                log::warn!("No schemes loaded, audio playback disabled");
                return;
            }
        }
    } else {
        cfg.scheme.clone()
    };

    switch_scheme(&scheme_name);
}

fn map_key_to_audio(vk_code: u16) -> Option<usize> {
    if let Some(cfg) = config::get_config() {
        if cfg.blocked_keys.contains(&vk_code) {
            return None;
        }
    }

    let keycode = vk_code as u8;
    unsafe {
        let keymap = KEYMAP.take();
        let first_n = FIRST_N_NON_UNIQUE;
        let result = match &keymap {
            Some(keymap) => {
                match keymap.get(&keycode) {
                    Some(idx) => Some(*idx as usize),
                    None => {
                        if first_n <= 0 {
                            None
                        } else {
                            let idx = keycode % (first_n as u8);
                            Some(idx as usize)
                        }
                    }
                }
            }
            None => None,
        };
        KEYMAP = keymap;
        result
    }
}

fn main() {
    logging::init();

    if let Err(e) = audio::init() {
        log::error!("{e}");
    }

    load_default_scheme();

    unsafe {
        let instance = GetModuleHandleW(None).unwrap();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            lpszClassName: w!("TickeysMain"),
            ..std::mem::zeroed()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("TickeysMain"),
            w!("Tickeys"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance.into()),
            None,
        );

        if hwnd.is_err() {
            log::error!("Failed to create main window");
            return;
        }

        let hwnd = hwnd.unwrap();

        if let Err(e) = keyboard::install(hwnd) {
            log::error!("{e}");
        }

        filter::install_foreground_hook(hwnd.0 as isize);

        tray::install_tray(hwnd.0 as isize, instance.0 as isize);

        log::info!("Tickeys Windows started");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_KEYDOWN_HOOK => {
            let vk_code = wparam.0 as u16;
            if let Some(index) = map_key_to_audio(vk_code) {
                audio::play_audio(index);
            }
            LRESULT::default()
        }
        WM_FOREGROUND_CHANGE => {
            filter::check_and_apply_mute();
            LRESULT::default()
        }
        WM_TRAYICON => {
            let evt = (lparam.0 as u32) & 0xFFFF;
            match evt {
                0x0203 => { // WM_LBUTTONDBLCLK
                    log::info!("Tray icon double-clicked");
                }
                0x0205 => { // WM_RBUTTONUP
                    tray::show_context_menu(hwnd.0 as isize);
                }
                _ => {}
            }
            LRESULT::default()
        }
        WM_COMMAND => {
            let cmd = (wparam.0 as u32) & 0xFFFF;
            match cmd {
                1001 => {
                    log::info!("Show settings");
                }
                1002 => {
                    let muted = audio::is_muted();
                    audio::set_mute(!muted);
                }
                1003 => {
                    let _ = DestroyWindow(hwnd);
                }
                _ => {}
            }
            LRESULT::default()
        }
        WM_DESTROY => {
            tray::remove_tray();
            filter::uninstall_foreground_hook();
            keyboard::uninstall();
            audio::shutdown();
            PostQuitMessage(0);
            LRESULT::default()
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
