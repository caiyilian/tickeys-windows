pub struct KeyboardMonitor;

impl KeyboardMonitor {
    pub fn new() -> Result<Self, String> {
        Ok(KeyboardMonitor)
    }

    pub fn set_enabled(&mut self, _enabled: bool) {}
}
