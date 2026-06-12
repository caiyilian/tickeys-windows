pub struct TrayIcon;

impl TrayIcon {
    pub fn new() -> Self {
        TrayIcon
    }

    pub fn show_notification(&self, _title: &str, _message: &str) {}
}
