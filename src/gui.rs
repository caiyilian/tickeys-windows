use std::sync::Mutex;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

const TBS_HORZ: u32 = 0x0000;
const TBS_AUTOTICKS: u32 = 0x0001;
const TBM_GETPOS: u32 = 0x0400;
const TBM_SETPOS: u32 = 0x0405;
const TBM_SETRANGE: u32 = 0x0406;
const TB_THUMBPOSITION: u32 = 4;
const TB_THUMBTRACK: u32 = 5;
const VOLUME_SLIDER_MIN: i32 = 0;
const VOLUME_SLIDER_MAX: i32 = 500;
const PITCH_SLIDER_MIN: i32 = 50;
const PITCH_SLIDER_MAX: i32 = 200;
const UDM_SETRANGE32: u32 = 0x046E;
const UDM_SETPOS32: u32 = 0x0470;
const UDM_SETBUDDY: u32 = 0x0469;
const EN_CHANGE: u32 = 0x0300;

#[repr(C)]
#[allow(non_snake_case)]
struct LVCOLUMNW {
    pub mask: u32,
    pub fmt: i32,
    pub cx: i32,
    pub pszText: *const u16,
    pub cchTextMax: i32,
    pub iSubItem: i32,
    pub iImage: i32,
    pub iOrder: i32,
    pub cxMin: i32,
    pub cxDefault: i32,
    pub cxIdeal: i32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct LVCITEMW {
    pub mask: u32,
    pub iItem: i32,
    pub iSubItem: i32,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: *mut u16,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: isize,
    pub iIndent: i32,
    pub iGroupId: i32,
    pub cColumns: u32,
    pub puColumns: *mut u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct OPENFILENAMEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: isize,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut u16,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const u16,
    pub lCustData: isize,
    pub lpfnHook: isize,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut std::ffi::c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

pub static SETTINGS_HWND: Mutex<isize> = Mutex::new(0);
static COMBOBOX_HWND: Mutex<isize> = Mutex::new(0);
static VOLUME_SLIDER_HWND: Mutex<isize> = Mutex::new(0);
static VOLUME_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static PITCH_SLIDER_HWND: Mutex<isize> = Mutex::new(0);
static PITCH_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static MAX_SOURCES_COMBO_HWND: Mutex<isize> = Mutex::new(0);
static MAX_SOURCES_EDITING: Mutex<bool> = Mutex::new(false);
static BLOCKED_KEYS_LISTVIEW_HWND: Mutex<isize> = Mutex::new(0);
pub static CAPTURING_KEY: Mutex<bool> = Mutex::new(false);
pub static PENDING_KEY: Mutex<u16> = Mutex::new(0);
static ADD_BUTTON_HWND: Mutex<isize> = Mutex::new(0);

fn vk_to_name(vk: u16) -> String {
    match vk {
        0x08 => "Backspace".to_string(),
        0x09 => "Tab".to_string(),
        0x0D => "Enter".to_string(),
        0x10 => "Shift".to_string(),
        0x11 => "Ctrl".to_string(),
        0x12 => "Alt".to_string(),
        0x13 => "Pause".to_string(),
        0x14 => "CapsLock".to_string(),
        0x1B => "Escape".to_string(),
        0x20 => "Space".to_string(),
        0x21 => "PageUp".to_string(),
        0x22 => "PageDown".to_string(),
        0x23 => "End".to_string(),
        0x24 => "Home".to_string(),
        0x25 => "Left".to_string(),
        0x26 => "Up".to_string(),
        0x27 => "Right".to_string(),
        0x28 => "Down".to_string(),
        0x2C => "PrintScreen".to_string(),
        0x2D => "Insert".to_string(),
        0x2E => "Delete".to_string(),
        0x5B => "LWin".to_string(),
        0x5C => "RWin".to_string(),
        0x60 => "Numpad0".to_string(),
        0x61 => "Numpad1".to_string(),
        0x62 => "Numpad2".to_string(),
        0x63 => "Numpad3".to_string(),
        0x64 => "Numpad4".to_string(),
        0x65 => "Numpad5".to_string(),
        0x66 => "Numpad6".to_string(),
        0x67 => "Numpad7".to_string(),
        0x68 => "Numpad8".to_string(),
        0x69 => "Numpad9".to_string(),
        0x6A => "Numpad*".to_string(),
        0x6B => "Numpad+".to_string(),
        0x6D => "Numpad-".to_string(),
        0x6E => "Numpad.".to_string(),
        0x6F => "Numpad/".to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0x90 => "NumLock".to_string(),
        0x91 => "ScrollLock".to_string(),
        0xA0 => "LShift".to_string(),
        0xA1 => "RShift".to_string(),
        0xA2 => "LCtrl".to_string(),
        0xA3 => "RCtrl".to_string(),
        0xA4 => "LAlt".to_string(),
        0xA5 => "RAlt".to_string(),
        0x30..=0x39 => format!("{}", vk - 0x30), // 0-9
        0x41..=0x5A => format!("{}", (vk as u8 + b'A' - 0x41) as char), // A-Z
        _ => format!("0x{:02X}", vk),
    }
}

pub struct SettingsWindow;

impl SettingsWindow {
    pub fn new() -> Self {
        SettingsWindow
    }

    pub fn show(&self) {
        unsafe {
            let hwnd = *SETTINGS_HWND.lock().unwrap();
            if hwnd != 0 {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_SHOW);
                // Use keybd_event trick to allow SetForegroundWindow
                keybd_event(0x12, 0, 0x0002, 0); // Alt up
                let _ = SetForegroundWindow(HWND(hwnd as *mut _));
                return;
            }

            let instance = GetModuleHandleW(None).unwrap();

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(settings_wnd_proc),
                hInstance: instance.into(),
                lpszClassName: w!("TickeysSettings"),
                ..std::mem::zeroed()
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST,
                w!("TickeysSettings"),
                w!("Tickeys \u{8BBE}\u{7F6E}"),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                400,
                300,
                None,
                None,
                Some(instance.into()),
                None,
            );

            if let Ok(hwnd) = hwnd {
                // Restore window position from config
                let config = crate::config::get_config();
                if let Some(ref cfg) = config {
                    if cfg.settings_x >= 0 && cfg.settings_y >= 0 {
                        let _ = SetWindowPos(
                            hwnd,
                            None,
                            cfg.settings_x,
                            cfg.settings_y,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOZORDER,
                        );
                    } else {
                        center_window(hwnd.0 as isize);
                    }
                } else {
                    center_window(hwnd.0 as isize);
                }

                create_controls(hwnd.0 as isize);
                let _ = ShowWindow(hwnd, SW_SHOW);
                *SETTINGS_HWND.lock().unwrap() = hwnd.0 as isize;
                log::info!("Settings window created");
            } else {
                log::error!("Failed to create settings window");
            }
        }
    }

    pub fn hide(&self) {
        unsafe {
            let hwnd = *SETTINGS_HWND.lock().unwrap();
            if hwnd != 0 {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_HIDE);
            }
        }
    }
}

fn center_window(hwnd: isize) {
    unsafe {
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        let mut rect = RECT::default();
        let _ = GetWindowRect(HWND(hwnd as *mut _), &mut rect);

        let window_width = rect.right - rect.left;
        let window_height = rect.bottom - rect.top;

        let x = (screen_width - window_width) / 2;
        let y = (screen_height - window_height) / 2;

        let _ = SetWindowPos(
            HWND(hwnd as *mut _),
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER,
        );
    }
}

fn make_trackbar_range(min: i32, max: i32) -> LPARAM {
    LPARAM(((min as u32 & 0xFFFF) | ((max as u32 & 0xFFFF) << 16)) as isize)
}

unsafe fn trackbar_scroll_position(
    slider_hwnd: isize,
    wparam: WPARAM,
    min: i32,
    max: i32,
) -> i32 {
    let scroll_code = (wparam.0 & 0xFFFF) as u32;
    let position = match scroll_code {
        TB_THUMBPOSITION | TB_THUMBTRACK => ((wparam.0 >> 16) & 0xFFFF) as i32,
        _ => {
            SendMessageW(
                HWND(slider_hwnd as *mut _),
                TBM_GETPOS,
                Some(WPARAM(0)),
                Some(LPARAM(0)),
            )
            .0 as i32
        }
    };

    position.clamp(min, max)
}

fn create_controls(hwnd: isize) {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap();

        let margin = 20;
        let label_w = 110;
        let ctrl_x = margin + label_w + 10;
        let ctrl_w = 260;
        let row_h = 28;
        let mut y = margin;

        // === 音效方案 ===
        let _scheme_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{97F3}\u{6548}\u{65B9}\u{6848}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(100 as *mut _)),
            Some(instance.into()),
            None,
        );

        let combo_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("COMBOBOX"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
            ctrl_x, y, ctrl_w, 200,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(1 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(combo_hwnd) = combo_hwnd {
            *COMBOBOX_HWND.lock().unwrap() = combo_hwnd.0 as isize;
            let schemes = crate::schemes::load_schemes();
            let current_scheme = crate::config::get_config()
                .map(|c| c.scheme)
                .unwrap_or_default();
            let mut selected_index: Option<i32> = None;
            for (index, scheme) in schemes.iter().enumerate() {
                let display_name: Vec<u16> = scheme.display_name.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = SendMessageW(combo_hwnd, CB_ADDSTRING, Some(WPARAM(0)), Some(LPARAM(display_name.as_ptr() as isize)));
                if scheme.name == current_scheme {
                    selected_index = Some(index as i32);
                }
            }
            if let Some(index) = selected_index {
                let _ = SendMessageW(combo_hwnd, CB_SETCURSEL, Some(WPARAM(index as usize)), Some(LPARAM(0)));
            }
        }

        y += row_h + 16;

        // === 音量 ===
        let _volume_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{97F3}\u{91CF}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(2 as *mut _)),
            Some(instance.into()),
            None,
        );

        let volume_slider_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_trackbar32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
            ctrl_x, y + 2, ctrl_w, row_h - 4,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(3 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(volume_slider_hwnd) = volume_slider_hwnd {
            *VOLUME_SLIDER_HWND.lock().unwrap() = volume_slider_hwnd.0 as isize;
            let current_volume = crate::config::get_config().map(|c| c.volume).unwrap_or(0.5);
            let slider_pos = ((current_volume * 100.0).round() as i32).clamp(VOLUME_SLIDER_MIN, VOLUME_SLIDER_MAX);
            let _ = SendMessageW(volume_slider_hwnd, TBM_SETRANGE, Some(WPARAM(1)), Some(make_trackbar_range(VOLUME_SLIDER_MIN, VOLUME_SLIDER_MAX)));
            let _ = SendMessageW(volume_slider_hwnd, TBM_SETPOS, Some(WPARAM(1)), Some(LPARAM(slider_pos as isize)));
            log::info!("Volume slider created with {}%", slider_pos);
        }

        y += row_h + 12;

        // === 音调 ===
        let _pitch_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{97F3}\u{8C03}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(4 as *mut _)),
            Some(instance.into()),
            None,
        );

        let pitch_slider_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_trackbar32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
            ctrl_x, y + 2, ctrl_w, row_h - 4,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(5 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(pitch_slider_hwnd) = pitch_slider_hwnd {
            *PITCH_SLIDER_HWND.lock().unwrap() = pitch_slider_hwnd.0 as isize;
            let current_pitch = crate::config::get_config().map(|c| c.pitch).unwrap_or(1.0);
            let slider_pos = ((current_pitch * 100.0).round() as i32).clamp(PITCH_SLIDER_MIN, PITCH_SLIDER_MAX);
            let _ = SendMessageW(pitch_slider_hwnd, TBM_SETRANGE, Some(WPARAM(1)), Some(make_trackbar_range(PITCH_SLIDER_MIN, PITCH_SLIDER_MAX)));
            let _ = SendMessageW(pitch_slider_hwnd, TBM_SETPOS, Some(WPARAM(1)), Some(LPARAM(slider_pos as isize)));
            log::info!("Pitch slider created with {:.1}", current_pitch);
        }

        y += row_h + 16;

        // === 同时播放数 ===
        let _max_sources_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{540C}\u{65F6}\u{64AD}\u{653E}\u{6570}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(6 as *mut _)),
            Some(instance.into()),
            None,
        );

        let current_max_sources = crate::config::get_config().map(|c| c.max_sources).unwrap_or(2);

        let max_sources_edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            None,
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(0x0080), // ES_AUTOHSCROLL
            ctrl_x, y + 2, 60, row_h - 4,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(7 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(max_sources_edit_hwnd) = max_sources_edit_hwnd {
            *MAX_SOURCES_COMBO_HWND.lock().unwrap() = max_sources_edit_hwnd.0 as isize;

            // Suppress EN_CHANGE during initialization to prevent re-entrant rebuild_player/update_config
            *MAX_SOURCES_EDITING.lock().unwrap() = true;

            let init_text = format!("{}", current_max_sources);
            let init_utf16: Vec<u16> = init_text.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = SetWindowTextW(max_sources_edit_hwnd, PCWSTR(init_utf16.as_ptr()));

            let spin_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("msctls_updown32"),
                None,
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0002), // UDS_SETBUDDYINT
                ctrl_x + 60, y + 2, 20, row_h - 4,
                Some(HWND(hwnd as *mut _)),
                Some(HMENU(16 as *mut _)),
                Some(instance.into()),
                None,
            );

            if let Ok(spin_hwnd) = spin_hwnd {
                let _ = SendMessageW(spin_hwnd, UDM_SETBUDDY, Some(WPARAM(max_sources_edit_hwnd.0 as usize)), Some(LPARAM(0)));
                let _ = SendMessageW(spin_hwnd, UDM_SETRANGE32, Some(WPARAM(2)), Some(LPARAM(20)));
                let _ = SendMessageW(spin_hwnd, UDM_SETPOS32, Some(WPARAM(1)), Some(LPARAM(current_max_sources as isize)));
            }

            *MAX_SOURCES_EDITING.lock().unwrap() = false;

            log::info!("Max sources spin control created with range 2-20, value={}", current_max_sources);
        }

        y += row_h + 16;

        // === 排除按键 ===
        let _blocked_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{6392}\u{9664}\u{6309}\u{952E}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(8 as *mut _)),
            Some(instance.into()),
            None,
        );

        let add_blocked_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{6DFB}\u{52A0}\u{6309}\u{952E}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001), // BS_PUSHBUTTON
            ctrl_x, y, 100, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(19 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(btn_hwnd) = add_blocked_btn {
            *ADD_BUTTON_HWND.lock().unwrap() = btn_hwnd.0 as isize;
        }

        let _remove_blocked_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{79FB}\u{9664}\u{6309}\u{952E}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001), // BS_PUSHBUTTON
            ctrl_x + 110, y, 100, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(20 as *mut _)),
            Some(instance.into()),
            None,
        );

        y += row_h + 8;

        // ListView for blocked keys
        let blocked_listview_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("SysListView32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0001 | 0x0002), // LVS_REPORT | LVS_SHOWSELALWAYS
            margin, y, 380, 140,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(18 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(listview_hwnd) = blocked_listview_hwnd {
            *BLOCKED_KEYS_LISTVIEW_HWND.lock().unwrap() = listview_hwnd.0 as isize;

            let column_text = "\u{6309}\u{952E}";
            let column_text_utf16: Vec<u16> = column_text.encode_utf16().chain(std::iter::once(0)).collect();
            let mut column = LVCOLUMNW {
                mask: 0x0001,
                pszText: column_text_utf16.as_ptr(),
                cx: 360,
                ..std::mem::zeroed()
            };
            let _ = SendMessageW(
                listview_hwnd,
                0x1061, // LVM_INSERTCOLUMNW
                Some(WPARAM(0)),
                Some(LPARAM(&mut column as *mut _ as isize)),
            );

            let current_config = crate::config::get_config();
            let blocked_keys = current_config.as_ref().map(|c| &c.blocked_keys).cloned().unwrap_or_default();

            for (index, &vk_code) in blocked_keys.iter().enumerate() {
                let key_name = vk_to_name(vk_code);
                let key_utf16: Vec<u16> = key_name.encode_utf16().chain(std::iter::once(0)).collect();
                let mut item = LVCITEMW {
                    mask: 0x0001,
                    iItem: index as i32,
                    pszText: key_utf16.as_ptr() as *mut _,
                    ..std::mem::zeroed()
                };
                let _ = SendMessageW(
                    listview_hwnd,
                    0x104D, // LVM_INSERTITEMW
                    Some(WPARAM(0)),
                    Some(LPARAM(&mut item as *mut _ as isize)),
                );
            }

            log::info!("Blocked keys ListView created with {} items", blocked_keys.len());
        }

        // Move y past the ListView
        y += 140 + 8;

        // Version label at bottom
        let version_text = format!("v{}", crate::consts::CURRENT_VERSION);
        let version_utf16: Vec<u16> = version_text.encode_utf16().chain(std::iter::once(0)).collect();

        let _version_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(version_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000010),
            margin, y, 380, 20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(14 as *mut _)),
            Some(instance.into()),
            None,
        );

        y += 24;

        // Website link
        let website_text = "\u{8BBF}\u{95EE}\u{5B98}\u{7F51}";
        let website_utf16: Vec<u16> = website_text.encode_utf16().chain(std::iter::once(0)).collect();

        let _website_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(website_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000010 | 0x00000200),
            margin, y, 380, 20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(15 as *mut _)),
            Some(instance.into()),
            None,
        );

        y += 30;

        // Adjust window size (add non-client area overhead for caption + borders)
        let _ = SetWindowPos(
            HWND(hwnd as *mut _),
            None,
            0, 0, 440,
            y + 47,
            SWP_NOMOVE | SWP_NOZORDER,
        );
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Prevent panics from crossing FFI boundary (causes 0xC0000005 / 0xC000041D)
    match std::panic::catch_unwind(|| {
        match msg {
            WM_COMMAND => {
                let code = (wparam.0 >> 16) & 0xFFFF;
                let id = wparam.0 & 0xFFFF;

                if id == 1 && code == 1 { // CBN_SELCHANGE for scheme selector
                let combo_hwnd = *COMBOBOX_HWND.lock().unwrap();
                if combo_hwnd != 0 {
                    let index = SendMessageW(
                        HWND(combo_hwnd as *mut _),
                        CB_GETCURSEL,
                        Some(WPARAM(0)),
                        Some(LPARAM(0)),
                    );

                    if index.0 != -1 {
                        let schemes = crate::schemes::load_schemes();
                        if let Some(scheme) = schemes.get(index.0 as usize) {
                            crate::switch_scheme(&scheme.name);
                            log::info!("Switched to scheme: {}", scheme.name);
                        }
                    }
                }
            } else if id == 7 && code == EN_CHANGE as usize { // EN_CHANGE for max sources Edit
                let max_sources_edit = *MAX_SOURCES_COMBO_HWND.lock().unwrap();
                if max_sources_edit != 0 {
                    // Guard against re-entrancy from SetWindowTextW
                    let mut editing = MAX_SOURCES_EDITING.lock().unwrap();
                    if *editing {
                        return LRESULT::default();
                    }
                    *editing = true;
                    drop(editing);

                    let mut text = [0u16; 16];
                    let len = GetWindowTextW(HWND(max_sources_edit as *mut _), &mut text);
                    if len > 0 {
                        let text_str = String::from_utf16_lossy(&text[..len as usize]);
                        log::info!("Max sources EN_CHANGE: text='{}'", text_str);
                        match text_str.parse::<i32>() {
                            Ok(val) => {
                                let clamped = val.clamp(2, 20) as usize;
                                let clean = format!("{}", clamped);
                                let clean_utf16: Vec<u16> = clean.encode_utf16().chain(std::iter::once(0)).collect();
                                let _ = SetWindowTextW(
                                    HWND(max_sources_edit as *mut _),
                                    PCWSTR(clean_utf16.as_ptr()),
                                );
                                crate::audio::rebuild_player(clamped);
                                if let Some(mut cfg) = crate::config::get_config() {
                                    cfg.max_sources = clamped;
                                    crate::config::update_config(&cfg);
                                }
                                log::info!("Max sources changed to {}", clamped);
                            }
                            Err(_) => {
                                let current = crate::config::get_config()
                                    .map(|c| c.max_sources)
                                    .unwrap_or(2);
                                let revert = format!("{}", current);
                                let revert_utf16: Vec<u16> = revert.encode_utf16().chain(std::iter::once(0)).collect();
                                let _ = SetWindowTextW(
                                    HWND(max_sources_edit as *mut _),
                                    PCWSTR(revert_utf16.as_ptr()),
                                );
                            }
                        }
                    }

                    *MAX_SOURCES_EDITING.lock().unwrap() = false;
                }
            } else if id == 19 && code == 0 { // Add blocked key button
                let pending = *PENDING_KEY.lock().unwrap();
                let capturing = *CAPTURING_KEY.lock().unwrap();

                if pending != 0 && !capturing {
                    // Confirm and add the captured key
                    if let Some(mut cfg) = crate::config::get_config() {
                        if !cfg.blocked_keys.contains(&pending) {
                            cfg.blocked_keys.push(pending);
                            cfg.blocked_keys.sort();
                            cfg.blocked_keys.dedup();
                            crate::config::update_config(&cfg);

                            // Update ListView
                            let listview_hwnd = *BLOCKED_KEYS_LISTVIEW_HWND.lock().unwrap();
                            if listview_hwnd != 0 {
                                let key_name = vk_to_name(pending);
                                let key_utf16: Vec<u16> = key_name.encode_utf16().chain(std::iter::once(0)).collect();
                                let mut item = LVCITEMW {
                                    mask: 0x0001,
                                    iItem: cfg.blocked_keys.len() as i32 - 1,
                                    pszText: key_utf16.as_ptr() as *mut _,
                                    ..std::mem::zeroed()
                                };
                                let _ = SendMessageW(
                                    HWND(listview_hwnd as *mut _),
                                    0x104D, // LVM_INSERTITEMW
                                    Some(WPARAM(0)),
                                    Some(LPARAM(&mut item as *mut _ as isize)),
                                );
                            }

                            log::info!("Added blocked key: {} (0x{:02X})", vk_to_name(pending), pending);
                        }
                    }
                    *PENDING_KEY.lock().unwrap() = 0;

                    // Reset button text
                    *CAPTURING_KEY.lock().unwrap() = false;
                    let btn_hwnd = *ADD_BUTTON_HWND.lock().unwrap();
                    if btn_hwnd != 0 {
                        let text = "\u{6DFB}\u{52A0}\u{6309}\u{952E}";
                        let text_utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = SetWindowTextW(
                            HWND(btn_hwnd as *mut _),
                            PCWSTR(text_utf16.as_ptr()),
                        );
                    }
                } else if !capturing {
                    // Start capturing
                    *CAPTURING_KEY.lock().unwrap() = true;
                    let btn_hwnd = *ADD_BUTTON_HWND.lock().unwrap();
                    if btn_hwnd != 0 {
                        let text = "\u{6309}\u{4E0B}\u{4EFB}\u{610F}\u{952E}...";
                        let text_utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                        let _ = SetWindowTextW(
                            HWND(btn_hwnd as *mut _),
                            PCWSTR(text_utf16.as_ptr()),
                        );
                    }
                    log::info!("Key capture started");
                }
            } else if id == 20 && code == 0 { // Remove blocked key button
                let listview_hwnd = *BLOCKED_KEYS_LISTVIEW_HWND.lock().unwrap();
                if listview_hwnd != 0 {
                    let mut selected_indices = Vec::new();
                    let mut index = -1;

                    loop {
                        index = SendMessageW(
                            HWND(listview_hwnd as *mut _),
                            0x100C, // LVM_GETNEXTITEM
                            Some(WPARAM(index as usize)),
                            Some(LPARAM(0x0002)), // LVNI_SELECTED
                        ).0 as i32;

                        if index == -1 {
                            break;
                        }
                        selected_indices.push(index);
                    }

                    if !selected_indices.is_empty() {
                        // Remove from ListView (reverse order)
                        for &idx in selected_indices.iter().rev() {
                            let _ = SendMessageW(
                                HWND(listview_hwnd as *mut _),
                                0x1008, // LVM_DELETEITEM
                                Some(WPARAM(idx as usize)),
                                Some(LPARAM(0)),
                            );
                        }

                        // Update config
                        if let Some(mut cfg) = crate::config::get_config() {
                            for &idx in selected_indices.iter().rev() {
                                if (idx as usize) < cfg.blocked_keys.len() {
                                    cfg.blocked_keys.remove(idx as usize);
                                }
                            }
                            crate::config::update_config(&cfg);
                            log::info!("Removed {} blocked keys", selected_indices.len());
                        }
                    }
                }
            }
            LRESULT::default()
        }
        WM_HSCROLL => {
            let volume_slider = *VOLUME_SLIDER_HWND.lock().unwrap();
            let pitch_slider = *PITCH_SLIDER_HWND.lock().unwrap();
            
            if volume_slider != 0 && lparam.0 as isize == volume_slider {
                let pos = trackbar_scroll_position(
                    volume_slider,
                    wparam,
                    VOLUME_SLIDER_MIN,
                    VOLUME_SLIDER_MAX,
                );

                let _ = SendMessageW(
                    HWND(volume_slider as *mut _),
                    TBM_SETPOS,
                    Some(WPARAM(1)),
                    Some(LPARAM(pos as isize)),
                );

                let volume = pos as f32 / 100.0;
                crate::audio::set_volume(volume);

                if let Some(mut cfg) = crate::config::get_config() {
                    cfg.volume = volume;
                    crate::config::update_config(&cfg);
                }

                let percentage = (volume * 100.0) as i32;
                let label_text = format!("\u{97F3}\u{91CF}: {}%", percentage);
                let label_hwnd = *VOLUME_LABEL_HWND.lock().unwrap();
                if label_hwnd != 0 {
                    let label_text_utf16: Vec<u16> = label_text.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = SetWindowTextW(
                        HWND(label_hwnd as *mut _),
                        PCWSTR(label_text_utf16.as_ptr()),
                    );
                }

                log::info!("Volume changed to {}%", percentage);
            } else if pitch_slider != 0 && lparam.0 as isize == pitch_slider {
                let pos = trackbar_scroll_position(
                    pitch_slider,
                    wparam,
                    PITCH_SLIDER_MIN,
                    PITCH_SLIDER_MAX,
                );

                let _ = SendMessageW(
                    HWND(pitch_slider as *mut _),
                    TBM_SETPOS,
                    Some(WPARAM(1)),
                    Some(LPARAM(pos as isize)),
                );

                let pitch = pos as f32 / 100.0;
                crate::audio::set_pitch(pitch);

                if let Some(mut cfg) = crate::config::get_config() {
                    cfg.pitch = pitch;
                    crate::config::update_config(&cfg);
                }

                let label_text = format!("\u{97F3}\u{8C03}: {:.1}", pitch);
                let label_hwnd = *PITCH_LABEL_HWND.lock().unwrap();
                if label_hwnd != 0 {
                    let label_text_utf16: Vec<u16> = label_text.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = SetWindowTextW(
                        HWND(label_hwnd as *mut _),
                        PCWSTR(label_text_utf16.as_ptr()),
                    );
                }

                log::info!("Pitch changed to {:.1}", pitch);
            }
            LRESULT::default()
        }
        crate::consts::WM_KEY_CAPTURED => {
            let vk_code = wparam.0 as u16;
            *PENDING_KEY.lock().unwrap() = vk_code;
            *CAPTURING_KEY.lock().unwrap() = false;

            let key_name = vk_to_name(vk_code);
            log::info!("WM_KEY_CAPTURED: vk_code=0x{:02X} ({}), key_name={}", vk_code, vk_code, key_name);

            // Update button text to show captured key
            let btn_hwnd = *ADD_BUTTON_HWND.lock().unwrap();
            if btn_hwnd != 0 {
                let btn_text = format!("\u{786E}\u{8BA4}\u{6DFB}\u{52A0} {}", key_name);
                let text_utf16: Vec<u16> = btn_text.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = SetWindowTextW(
                    HWND(btn_hwnd as *mut _),
                    PCWSTR(text_utf16.as_ptr()),
                );
            }

            LRESULT::default()
        }
        WM_NOTIFY => {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MOVE => {
            // Save window position
            let mut rect = RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect);
            if let Some(mut cfg) = crate::config::get_config() {
                cfg.settings_x = rect.left;
                cfg.settings_y = rect.top;
                crate::config::update_config(&cfg);
            }
            LRESULT::default()
        }
        WM_CLOSE => {
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT::default()
        }
        WM_DESTROY => {
            *SETTINGS_HWND.lock().unwrap() = 0;
            *COMBOBOX_HWND.lock().unwrap() = 0;
            *VOLUME_SLIDER_HWND.lock().unwrap() = 0;
            *VOLUME_LABEL_HWND.lock().unwrap() = 0;
            *PITCH_SLIDER_HWND.lock().unwrap() = 0;
            *PITCH_LABEL_HWND.lock().unwrap() = 0;
            *MAX_SOURCES_COMBO_HWND.lock().unwrap() = 0;
            *MAX_SOURCES_EDITING.lock().unwrap() = false;
            *BLOCKED_KEYS_LISTVIEW_HWND.lock().unwrap() = 0;
            LRESULT::default()
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    }) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Panic in settings_wnd_proc(msg=0x{:X}): {:?}", msg, e);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

#[link(name = "user32")]
extern "system" {
    fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
}

#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
}
