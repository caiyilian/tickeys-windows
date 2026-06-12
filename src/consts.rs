pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const APP_NAME: &str = "Tickeys";
pub const MUTEX_NAME: &str = "Global\\Tickeys_SingleInstance";

pub const WEBSITE: &str = "http://www.yingdev.com/projects/tickeys";
pub const DONATE_URL: &str = "http://www.yingdev.com/home/donate";

// --- Windows VK codes ---
pub const VK_A: u32 = 0x41;
pub const VK_Q: u32 = 0x51;
pub const VK_Z: u32 = 0x5A;
pub const VK_1: u32 = 0x31;
pub const VK_2: u32 = 0x32;
pub const VK_3: u32 = 0x33;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SPACE: u32 = 0x20;
pub const VK_BACK: u32 = 0x08;

// --- Custom window messages ---
pub const WM_TRAYICON: u32 = 0x0401;   // WM_USER + 1
pub const WM_KEYDOWN_HOOK: u32 = 0x0402;  // WM_USER + 2
pub const WM_FOREGROUND_CHANGE: u32 = 0x0403;  // WM_USER + 3
pub const WM_SHOW_SETTINGS: u32 = 0x0410;  // WM_USER + 10 (for shortcut key)

// --- Power event constants ---
pub const WM_POWERBROADCAST: u32 = 0x0218;
pub const PBT_APMRESUMEAUTOMATIC: u32 = 0x0012;
pub const PBT_APMRESUMESUSPEND: u32 = 0x0007;
pub const PBT_APMSUSPEND: u32 = 0x0004;

// --- DPI awareness constants ---
pub const PROCESS_PER_MONITOR_DPI_AWARE: u32 = 2;

// --- Shortcut key sequences ---
// QAZ123 (macOS: [12,0,6,18,19,20]) mapped to Windows VK codes
pub const OPEN_SETTINGS_KEY_SEQ: &[&[u32]] = &[
    &[VK_Q, VK_A, VK_Z, VK_1, VK_2, VK_3],         // QAZ123 (main keyboard)
    &[VK_Q, VK_A, VK_Z, 0x61, 0x62, 0x63],          // QAZ123 (numpad)
];
