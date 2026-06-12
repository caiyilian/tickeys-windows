pub struct SimpleAudioPlayer;

impl SimpleAudioPlayer {
    pub fn new(_max_sources: usize) -> Self {
        SimpleAudioPlayer
    }

    pub fn play(&mut self, _index: usize) {}

    pub fn set_gain(&mut self, _gain: f32) {}

    pub fn set_pitch(&mut self, _pitch: f32) {}

    pub fn rebuild(&mut self, _new_count: usize) {}
}
