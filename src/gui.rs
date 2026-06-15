use std::sync::Mutex;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
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
const MAX_SOURCES_MIN: i32 = 2;
const MAX_SOURCES_MAX: i32 = 20;
const DEBOUNCE_MIN: i32 = 10;
const DEBOUNCE_MAX: i32 = 500;
const UDM_SETRANGE32: u32 = 0x046F;
const UDM_SETPOS32: u32 = 0x0471;
const UDM_SETBUDDY: u32 = 0x0469;
const UDN_DELTAPOS: u32 = 0xFFFF_FD2E;
const EN_CHANGE: u32 = 0x0300;
const ES_AUTOHSCROLL: u32 = 0x0080;
const ES_NUMBER: u32 = 0x2000;
const UDS_ARROWKEYS: u32 = 0x0020;
const UDS_NOTHOUSANDS: u32 = 0x0080;

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
struct NMHDR_LOCAL {
    pub hwndFrom: HWND,
    pub idFrom: usize,
    pub code: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct NMUPDOWN_LOCAL {
    pub hdr: NMHDR_LOCAL,
    pub iPos: i32,
    pub iDelta: i32,
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
static PEAK_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static DEBOUNCE_EDIT_HWND: Mutex<isize> = Mutex::new(0);
static DEBOUNCE_EDITING: Mutex<bool> = Mutex::new(false);
static UI_FONT: Mutex<isize> = Mutex::new(0);
static UI_TITLE_FONT: Mutex<isize> = Mutex::new(0);

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn create_ui_font(height: i32, weight: i32) -> HFONT {
    let face = wide("Segoe UI");
    CreateFontW(
        height,
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        DEFAULT_QUALITY,
        (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
        PCWSTR(face.as_ptr()),
    )
}

unsafe fn settings_font() -> HFONT {
    let mut stored = UI_FONT.lock().unwrap();
    if *stored == 0 {
        *stored = create_ui_font(-14, FW_NORMAL.0 as i32).0 as isize;
    }
    HFONT(*stored as *mut _)
}

unsafe fn settings_title_font() -> HFONT {
    let mut stored = UI_TITLE_FONT.lock().unwrap();
    if *stored == 0 {
        *stored = create_ui_font(-19, FW_SEMIBOLD.0 as i32).0 as isize;
    }
    HFONT(*stored as *mut _)
}

unsafe fn destroy_ui_fonts() {
    let mut font = UI_FONT.lock().unwrap();
    if *font != 0 {
        let _ = DeleteObject(HGDIOBJ(*font as *mut _));
        *font = 0;
    }
    let mut title_font = UI_TITLE_FONT.lock().unwrap();
    if *title_font != 0 {
        let _ = DeleteObject(HGDIOBJ(*title_font as *mut _));
        *title_font = 0;
    }
}

unsafe fn set_control_font(hwnd: HWND, font: HFONT) {
    if !font.0.is_null() {
        let _ = SendMessageW(
            hwnd,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
}

unsafe fn apply_font_to(control: &Result<HWND>, font: HFONT) {
    if let Ok(hwnd) = control {
        set_control_font(*hwnd, font);
    }
}

fn clamp_max_sources(value: i32) -> usize {
    value.clamp(MAX_SOURCES_MIN, MAX_SOURCES_MAX) as usize
}

fn config_max_sources() -> usize {
    crate::config::get_config()
        .map(|c| clamp_max_sources(c.max_sources as i32))
        .unwrap_or(MAX_SOURCES_MIN as usize)
}

unsafe fn set_max_sources_text(edit_hwnd: isize, value: usize) {
    let text = wide(&value.to_string());
    let _ = SetWindowTextW(HWND(edit_hwnd as *mut _), PCWSTR(text.as_ptr()));
}

unsafe fn read_max_sources_text(edit_hwnd: isize) -> Option<usize> {
    let mut text = [0u16; 16];
    let len = GetWindowTextW(HWND(edit_hwnd as *mut _), &mut text);
    if len <= 0 {
        return None;
    }
    let text_str = String::from_utf16_lossy(&text[..len as usize]);
    text_str.trim().parse::<i32>().ok().map(clamp_max_sources)
}

unsafe fn begin_max_sources_edit() -> bool {
    let mut editing = MAX_SOURCES_EDITING.lock().unwrap();
    if *editing {
        return false;
    }
    *editing = true;
    true
}

fn end_max_sources_edit() {
    *MAX_SOURCES_EDITING.lock().unwrap() = false;
}

unsafe fn commit_max_sources(edit_hwnd: isize, value: usize) {
    let previous = config_max_sources();
    set_max_sources_text(edit_hwnd, value);
    if value != previous {
        crate::audio::rebuild_player(value);
        if let Some(mut cfg) = crate::config::get_config() {
            cfg.max_sources = value;
            crate::config::update_config(&cfg);
        }
        log::info!("Max sources changed to {}", value);
    }
}

fn clamp_debounce_ms(value: i32) -> u32 {
    value.clamp(DEBOUNCE_MIN, DEBOUNCE_MAX) as u32
}

fn config_debounce_ms() -> u32 {
    crate::config::get_config()
        .map(|c| clamp_debounce_ms(c.key_debounce_ms as i32))
        .unwrap_or(DEBOUNCE_MIN as u32)
}

unsafe fn set_debounce_text(edit_hwnd: isize, value: u32) {
    let text = wide(&value.to_string());
    let _ = SetWindowTextW(HWND(edit_hwnd as *mut _), PCWSTR(text.as_ptr()));
}

unsafe fn read_debounce_text(edit_hwnd: isize) -> Option<u32> {
    let mut text = [0u16; 16];
    let len = GetWindowTextW(HWND(edit_hwnd as *mut _), &mut text);
    if len <= 0 {
        return None;
    }
    let text_str = String::from_utf16_lossy(&text[..len as usize]);
    text_str.trim().parse::<i32>().ok().map(clamp_debounce_ms)
}

unsafe fn begin_debounce_edit() -> bool {
    let mut editing = DEBOUNCE_EDITING.lock().unwrap();
    if *editing {
        return false;
    }
    *editing = true;
    true
}

fn end_debounce_edit() {
    *DEBOUNCE_EDITING.lock().unwrap() = false;
}

unsafe fn commit_debounce_ms(edit_hwnd: isize, value: u32) {
    let previous = config_debounce_ms();
    set_debounce_text(edit_hwnd, value);
    if value != previous {
        crate::keyboard::set_debounce_ms(value);
        if let Some(mut cfg) = crate::config::get_config() {
            cfg.key_debounce_ms = value;
            crate::config::update_config(&cfg);
        }
        log::info!("Key debounce changed to {}ms", value);
    }
}

unsafe fn handle_debounce_edit_change() -> LRESULT {
    let edit_hwnd = *DEBOUNCE_EDIT_HWND.lock().unwrap();
    if edit_hwnd == 0 || !begin_debounce_edit() {
        return LRESULT::default();
    }

    if let Some(value) = read_debounce_text(edit_hwnd) {
        commit_debounce_ms(edit_hwnd, value);
    }

    end_debounce_edit();
    LRESULT::default()
}

unsafe fn handle_debounce_spin_delta(lparam: LPARAM) -> Option<LRESULT> {
    if lparam.0 == 0 {
        return None;
    }
    let updown = &*(lparam.0 as *const NMUPDOWN_LOCAL);
    if updown.hdr.code != UDN_DELTAPOS || updown.hdr.idFrom != 11 {
        return None;
    }

    let edit_hwnd = *DEBOUNCE_EDIT_HWND.lock().unwrap();
    if edit_hwnd == 0 || !begin_debounce_edit() {
        return Some(LRESULT(1));
    }

    let current = read_debounce_text(edit_hwnd).unwrap_or_else(config_debounce_ms);
    let next = clamp_debounce_ms(current as i32 + updown.iDelta);
    commit_debounce_ms(edit_hwnd, next);
    let _ = SendMessageW(
        updown.hdr.hwndFrom,
        UDM_SETPOS32,
        Some(WPARAM(0)),
        Some(LPARAM(next as isize)),
    );

    end_debounce_edit();
    Some(LRESULT(1))
}

unsafe fn handle_max_sources_edit_change() -> LRESULT {
    let edit_hwnd = *MAX_SOURCES_COMBO_HWND.lock().unwrap();
    if edit_hwnd == 0 || !begin_max_sources_edit() {
        return LRESULT::default();
    }

    if let Some(value) = read_max_sources_text(edit_hwnd) {
        commit_max_sources(edit_hwnd, value);
    }

    end_max_sources_edit();
    LRESULT::default()
}

unsafe fn handle_max_sources_spin_delta(lparam: LPARAM) -> Option<LRESULT> {
    if lparam.0 == 0 {
        return None;
    }
    let updown = &*(lparam.0 as *const NMUPDOWN_LOCAL);
    if updown.hdr.code != UDN_DELTAPOS || updown.hdr.idFrom != 16 {
        return None;
    }

    let edit_hwnd = *MAX_SOURCES_COMBO_HWND.lock().unwrap();
    if edit_hwnd == 0 || !begin_max_sources_edit() {
        return Some(LRESULT(1));
    }

    let current = read_max_sources_text(edit_hwnd).unwrap_or_else(config_max_sources);
    let next = clamp_max_sources(current as i32 + updown.iDelta);
    commit_max_sources(edit_hwnd, next);
    let _ = SendMessageW(
        updown.hdr.hwndFrom,
        UDM_SETPOS32,
        Some(WPARAM(0)),
        Some(LPARAM(next as isize)),
    );

    end_max_sources_edit();
    Some(LRESULT(1))
}

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
                470,
                560,
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

unsafe fn trackbar_scroll_position(slider_hwnd: isize, wparam: WPARAM, min: i32, max: i32) -> i32 {
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
        let parent = HWND(hwnd as *mut _);
        let ui_font = settings_font();
        let title_font = settings_title_font();

        // ======== 布局参数，改这里调整位置 ========
        let mut client_rect = RECT::default();
        let _ = GetClientRect(parent, &mut client_rect);
        let client_w = (client_rect.right - client_rect.left) as i32;

        let group_w = 436;      // 分组框宽度
        let group_x = (client_w - group_w) / 2;  // 分组框居中 x

        let margin = 22;        // 左边距
        let label_w = 104;      // 标签宽度
        let ctrl_x = margin + label_w + 14;  // 控件起始 x（标签右侧）
        let ctrl_w = 286;       // 控件宽度
        let row_h = 28;         // 每行高度
        let mut y = 16;         // 当前 y 坐标（从顶部开始）
        // ==========================================

        // --- 标题 "Tickeys 设置" ---
        // y=16, 高度=26
        let title_text = wide("Tickeys \u{8BBE}\u{7F6E}");
        let title_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(title_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001),  // SS_CENTER
            margin, y, 416, 26,  // x, y, 宽, 高
            Some(parent),
            Some(HMENU(101 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&title_label, title_font);

        // --- 副标题 "调整键盘音效、播放性能和排除按键" ---
        y += 28;  // 标题高度 + 间距
        // y=44, 高度=20
        let subtitle_text = wide("\u{8C03}\u{6574}\u{952E}\u{76D8}\u{97F3}\u{6548}\u{3001}\u{64AD}\u{653E}\u{6027}\u{80FD}\u{548C}\u{6392}\u{9664}\u{6309}\u{952E}");
        let subtitle_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(subtitle_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001),
            margin, y, 416, 20,  // x, y, 宽, 高
            Some(parent),
            Some(HMENU(102 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&subtitle_label, ui_font);

        y += 50;  // 副标题高度 + 间距

        // --- "音效" 分组框 ---
        // y=80, 分组框边框在 y-12=68, 高度=132
        let audio_group_text = wide("\u{97F3}\u{6548}");
        let audio_group = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            PCWSTR(audio_group_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
            group_x, y - 20, group_w, 140,  // 居中, y偏移, 宽, 高
            Some(parent),
            Some(HMENU(103 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&audio_group, ui_font);

        // --- 音效方案 标签 + 下拉框 ---
        // y=80
        let _scheme_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{97F3}\u{6548}\u{65B9}\u{6848}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,  // x=22, y=80, 宽=104, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(100 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&_scheme_label, ui_font);

        let combo_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("COMBOBOX"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
            ctrl_x, y, ctrl_w, 200,  // x=140, y=80, 宽=286, 下拉高度=200
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(1 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&combo_hwnd, ui_font);

        if let Ok(combo_hwnd) = combo_hwnd {
            *COMBOBOX_HWND.lock().unwrap() = combo_hwnd.0 as isize;
            let schemes = crate::schemes::load_schemes();
            let current_scheme = crate::config::get_config()
                .map(|c| c.scheme)
                .unwrap_or_default();
            let mut selected_index: Option<i32> = None;
            for (index, scheme) in schemes.iter().enumerate() {
                let display_name: Vec<u16> = scheme
                    .display_name
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                let _ = SendMessageW(
                    combo_hwnd,
                    CB_ADDSTRING,
                    Some(WPARAM(0)),
                    Some(LPARAM(display_name.as_ptr() as isize)),
                );
                if scheme.name == current_scheme {
                    selected_index = Some(index as i32);
                }
            }
            if let Some(index) = selected_index {
                let _ = SendMessageW(
                    combo_hwnd,
                    CB_SETCURSEL,
                    Some(WPARAM(index as usize)),
                    Some(LPARAM(0)),
                );
            }
        }

        y += row_h + 16;  // 音效方案行高度 + 间距

        // --- 音量 标签 + 滑块 ---
        // y=124
        let current_volume = crate::config::get_config().map(|c| c.volume).unwrap_or(0.5);
        let volume_label_text = wide(&format!(
            "\u{97F3}\u{91CF}: {}%",
            (current_volume * 100.0).round() as i32
        ));
        let _volume_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(volume_label_text.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,  // x=22, y=124, 宽=104, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(2 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&_volume_label, ui_font);
        if let Ok(label_hwnd) = _volume_label {
            *VOLUME_LABEL_HWND.lock().unwrap() = label_hwnd.0 as isize;
        }

        let volume_slider_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_trackbar32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
            ctrl_x, y + 2, ctrl_w, row_h - 4,  // x=140, y=126, 宽=286, 高=24
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(3 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&volume_slider_hwnd, ui_font);

        if let Ok(volume_slider_hwnd) = volume_slider_hwnd {
            *VOLUME_SLIDER_HWND.lock().unwrap() = volume_slider_hwnd.0 as isize;
            let slider_pos = ((current_volume * 100.0).round() as i32)
                .clamp(VOLUME_SLIDER_MIN, VOLUME_SLIDER_MAX);
            let _ = SendMessageW(
                volume_slider_hwnd,
                TBM_SETRANGE,
                Some(WPARAM(1)),
                Some(make_trackbar_range(VOLUME_SLIDER_MIN, VOLUME_SLIDER_MAX)),
            );
            let _ = SendMessageW(
                volume_slider_hwnd,
                TBM_SETPOS,
                Some(WPARAM(1)),
                Some(LPARAM(slider_pos as isize)),
            );
            log::info!("Volume slider created with {}%", slider_pos);
        }

        y += row_h + 12;  // 音量行高度 + 间距（比其他行小一点）

        // --- 音调 标签 + 滑块 ---
        // y=164
        let current_pitch = crate::config::get_config().map(|c| c.pitch).unwrap_or(1.0);
        let pitch_label_text = wide(&format!("\u{97F3}\u{8C03}: {:.1}", current_pitch));
        let _pitch_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(pitch_label_text.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,  // x=22, y=164, 宽=104, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(4 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&_pitch_label, ui_font);
        if let Ok(label_hwnd) = _pitch_label {
            *PITCH_LABEL_HWND.lock().unwrap() = label_hwnd.0 as isize;
        }

        let pitch_slider_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_trackbar32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
            ctrl_x, y + 2, ctrl_w, row_h - 4,  // x=140, y=166, 宽=286, 高=24
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(5 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&pitch_slider_hwnd, ui_font);

        if let Ok(pitch_slider_hwnd) = pitch_slider_hwnd {
            *PITCH_SLIDER_HWND.lock().unwrap() = pitch_slider_hwnd.0 as isize;
            let slider_pos =
                ((current_pitch * 100.0).round() as i32).clamp(PITCH_SLIDER_MIN, PITCH_SLIDER_MAX);
            let _ = SendMessageW(
                pitch_slider_hwnd,
                TBM_SETRANGE,
                Some(WPARAM(1)),
                Some(make_trackbar_range(PITCH_SLIDER_MIN, PITCH_SLIDER_MAX)),
            );
            let _ = SendMessageW(
                pitch_slider_hwnd,
                TBM_SETPOS,
                Some(WPARAM(1)),
                Some(LPARAM(slider_pos as isize)),
            );
            log::info!("Pitch slider created with {:.1}", current_pitch);
        }

        y += row_h + 40;  // 音调行高度 + 间距

        // --- "播放性能" 分组框 ---
        // 分组框边框在 y-30, 高度=145（容纳三行）
        let playback_group_text = wide("\u{64AD}\u{653E}\u{6027}\u{80FD}");
        let playback_group = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            PCWSTR(playback_group_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
            group_x, y - 30, group_w, 145,
            Some(parent),
            Some(HMENU(104 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&playback_group, ui_font);

        // --- 同时播放数 标签 + 输入框 + 箭头 ---
        // y=208
        let _max_sources_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{540C}\u{65F6}\u{64AD}\u{653E}\u{6570}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,  // x=22, y=208, 宽=104, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(6 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&_max_sources_label, ui_font);

        let current_max_sources = config_max_sources();

        let max_sources_edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            None,
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL | ES_NUMBER),
            ctrl_x, y + 2, 60, row_h - 4,  // x=140, y=210, 宽=60, 高=24
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(7 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&max_sources_edit_hwnd, ui_font);

        if let Ok(max_sources_edit_hwnd) = max_sources_edit_hwnd {
            *MAX_SOURCES_COMBO_HWND.lock().unwrap() = max_sources_edit_hwnd.0 as isize;

            *MAX_SOURCES_EDITING.lock().unwrap() = true;
            set_max_sources_text(max_sources_edit_hwnd.0 as isize, current_max_sources);

            let spin_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("msctls_updown32"),
                None,
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(UDS_ARROWKEYS | UDS_NOTHOUSANDS),
                ctrl_x + 60, y + 2, 20, row_h - 4,  // x=200, y=210, 宽=20, 高=24
                Some(HWND(hwnd as *mut _)),
                Some(HMENU(16 as *mut _)),
                Some(instance.into()),
                None,
            );
            apply_font_to(&spin_hwnd, ui_font);

            if let Ok(spin_hwnd) = spin_hwnd {
                let _ = SendMessageW(
                    spin_hwnd,
                    UDM_SETBUDDY,
                    Some(WPARAM(max_sources_edit_hwnd.0 as usize)),
                    Some(LPARAM(0)),
                );
                let _ = SendMessageW(
                    spin_hwnd,
                    UDM_SETRANGE32,
                    Some(WPARAM(MAX_SOURCES_MIN as usize)),
                    Some(LPARAM(MAX_SOURCES_MAX as isize)),
                );
                let _ = SendMessageW(
                    spin_hwnd,
                    UDM_SETPOS32,
                    Some(WPARAM(0)),
                    Some(LPARAM(current_max_sources as isize)),
                );
            }

            *MAX_SOURCES_EDITING.lock().unwrap() = false;

            log::info!(
                "Max sources spin control created with range 2-20, edit_text={}",
                current_max_sources
            );
        }

        // --- 同时播放数 提示文本 ---
        // x=232, y=210
        let max_sources_hint = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{6700}\u{5927}20\u{FF0C}\u{6700}\u{5C0F}2"),
            WS_CHILD | WS_VISIBLE,
            ctrl_x + 92, y + 2, 200, row_h - 4,  // x=232, y=210, 宽=220, 高=24
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(105 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&max_sources_hint, ui_font);

        // --- 峰值同时播放数 ---
        // 第二行：y + row_h + 6
        let peak_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{5CF0}\u{503C}\u{540C}\u{65F6}\u{64AD}\u{653E}\u{6570}: 0"),
            WS_CHILD | WS_VISIBLE,
            margin, y + row_h + 6, 250, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(107 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&peak_label, ui_font);
        *PEAK_LABEL_HWND.lock().unwrap() = peak_label.as_ref().map_or(0, |h| h.0 as isize);

        // --- 按键防抖(ms) 标签 + 输入框 + 箭头 ---
        // 第三行：y + 2*(row_h + 6) + 2
        let debounce_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{6309}\u{952E}\u{9632}\u{6296}(ms)"),
            WS_CHILD | WS_VISIBLE,
            margin, y + 2 * (row_h + 6) + 2, label_w, row_h,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(11 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&debounce_label, ui_font);

        let current_debounce = config_debounce_ms();

        let debounce_edit_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("EDIT"),
            None,
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL | ES_NUMBER),
            ctrl_x, y + 2 * (row_h + 6) + 4, 60, row_h - 4,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(12 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&debounce_edit_hwnd, ui_font);

        if let Ok(debounce_edit_hwnd) = debounce_edit_hwnd {
            *DEBOUNCE_EDIT_HWND.lock().unwrap() = debounce_edit_hwnd.0 as isize;

            *DEBOUNCE_EDITING.lock().unwrap() = true;
            set_debounce_text(debounce_edit_hwnd.0 as isize, current_debounce);

            let spin_hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("msctls_updown32"),
                None,
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(UDS_ARROWKEYS | UDS_NOTHOUSANDS),
                ctrl_x + 60, y + 2 * (row_h + 6) + 4, 20, row_h - 4,
                Some(HWND(hwnd as *mut _)),
                Some(HMENU(13 as *mut _)),
                Some(instance.into()),
                None,
            );
            apply_font_to(&spin_hwnd, ui_font);

            if let Ok(spin_hwnd) = spin_hwnd {
                let _ = SendMessageW(
                    spin_hwnd,
                    UDM_SETBUDDY,
                    Some(WPARAM(debounce_edit_hwnd.0 as usize)),
                    Some(LPARAM(0)),
                );
                let _ = SendMessageW(
                    spin_hwnd,
                    UDM_SETRANGE32,
                    Some(WPARAM(DEBOUNCE_MIN as usize)),
                    Some(LPARAM(DEBOUNCE_MAX as isize)),
                );
                let _ = SendMessageW(
                    spin_hwnd,
                    UDM_SETPOS32,
                    Some(WPARAM(0)),
                    Some(LPARAM(current_debounce as isize)),
                );
            }

            *DEBOUNCE_EDITING.lock().unwrap() = false;

            log::info!(
                "Debounce spin control created with range {}-{}, value={}",
                DEBOUNCE_MIN, DEBOUNCE_MAX, current_debounce
            );
        }

        // --- 按键防抖 提示文本 ---
        let debounce_hint = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{8303}\u{56F4}10-500"),
            WS_CHILD | WS_VISIBLE,
            ctrl_x + 92, y + 2 * (row_h + 6) + 4, 200, row_h - 4,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(108 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&debounce_hint, ui_font);

        y += row_h * 3 + 60;  // 三行高度 + 间距

        // --- "排除按键" 分组框 ---
        // y=252, 分组框边框在 y-52=200, 高度=276
        let blocked_group_text = wide("\u{6392}\u{9664}\u{6309}\u{952E}");
        let blocked_group = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            PCWSTR(blocked_group_text.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_GROUPBOX as u32),
            group_x, y - 22, group_w, 210,  // 居中, y偏移, 宽, 高
            Some(parent),
            Some(HMENU(106 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&blocked_group, ui_font);

        // --- 排除按键 标签 + 按钮 ---
        // y=252
        let _blocked_label = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("\u{6392}\u{9664}\u{6309}\u{952E}"),
            WS_CHILD | WS_VISIBLE,
            margin, y, label_w, row_h,  // x=22, y=252, 宽=104, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(8 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&_blocked_label, ui_font);

        let add_blocked_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{6DFB}\u{52A0}\u{6309}\u{952E}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001), // BS_PUSHBUTTON
            ctrl_x, y, 100, row_h,  // x=140, y=252, 宽=100, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(19 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&add_blocked_btn, ui_font);

        if let Ok(btn_hwnd) = add_blocked_btn {
            *ADD_BUTTON_HWND.lock().unwrap() = btn_hwnd.0 as isize;
        }

        let _remove_blocked_btn = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{79FB}\u{9664}\u{6309}\u{952E}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001), // BS_PUSHBUTTON
            ctrl_x + 110, y, 100, row_h,  // x=250, y=252, 宽=100, 高=28
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(20 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&_remove_blocked_btn, ui_font);

        y += row_h + 8;  // 按钮行高度 + 间距

        // --- ListView（排除按键列表）---
        // y=288
        let blocked_listview_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("SysListView32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0001 | 0x0002), // LVS_REPORT | LVS_SHOWSELALWAYS
            margin, y, 404, 140,  // x=22, y=288, 宽=404, 高=140
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(18 as *mut _)),
            Some(instance.into()),
            None,
        );
        apply_font_to(&blocked_listview_hwnd, ui_font);

        if let Ok(listview_hwnd) = blocked_listview_hwnd {
            *BLOCKED_KEYS_LISTVIEW_HWND.lock().unwrap() = listview_hwnd.0 as isize;

            let column_text = "\u{6309}\u{952E}";
            let column_text_utf16: Vec<u16> = column_text
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let mut column = LVCOLUMNW {
                mask: 0x0001,
                pszText: column_text_utf16.as_ptr(),
                cx: 382,
                ..std::mem::zeroed()
            };
            let _ = SendMessageW(
                listview_hwnd,
                0x1061, // LVM_INSERTCOLUMNW
                Some(WPARAM(0)),
                Some(LPARAM(&mut column as *mut _ as isize)),
            );

            let current_config = crate::config::get_config();
            let blocked_keys = current_config
                .as_ref()
                .map(|c| &c.blocked_keys)
                .cloned()
                .unwrap_or_default();

            for (index, &vk_code) in blocked_keys.iter().enumerate() {
                let key_name = vk_to_name(vk_code);
                let key_utf16: Vec<u16> =
                    key_name.encode_utf16().chain(std::iter::once(0)).collect();
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

            log::info!(
                "Blocked keys ListView created with {} items",
                blocked_keys.len()
            );
        }

        // Move y past the ListView
        y += 210;

        // Adjust window size (add non-client area overhead for caption + borders)
        let _ = SetWindowPos(
            HWND(hwnd as *mut _),
            None,
            0,
            0,
            470,
            y,
            SWP_NOMOVE | SWP_NOZORDER,
        );

        // Start timer to refresh peak sources label every second
        let _ = SetTimer(Some(HWND(hwnd as *mut _)), 200usize, 1000u32, None);
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

                if id == 1 && code == 1 {
                    // CBN_SELCHANGE for scheme selector
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
                } else if id == 7 && code == EN_CHANGE as usize {
                    // EN_CHANGE for max sources Edit
                    return handle_max_sources_edit_change();
                } else if id == 12 && code == EN_CHANGE as usize {
                    // EN_CHANGE for debounce Edit
                    return handle_debounce_edit_change();
                } else if id == 19 && code == 0 {
                    // Add blocked key button
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
                                    let key_utf16: Vec<u16> =
                                        key_name.encode_utf16().chain(std::iter::once(0)).collect();
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

                                log::info!(
                                    "Added blocked key: {} (0x{:02X})",
                                    vk_to_name(pending),
                                    pending
                                );
                            }
                        }
                        *PENDING_KEY.lock().unwrap() = 0;

                        // Reset button text
                        *CAPTURING_KEY.lock().unwrap() = false;
                        let btn_hwnd = *ADD_BUTTON_HWND.lock().unwrap();
                        if btn_hwnd != 0 {
                            let text = "\u{6DFB}\u{52A0}\u{6309}\u{952E}";
                            let text_utf16: Vec<u16> =
                                text.encode_utf16().chain(std::iter::once(0)).collect();
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
                            let text_utf16: Vec<u16> =
                                text.encode_utf16().chain(std::iter::once(0)).collect();
                            let _ = SetWindowTextW(
                                HWND(btn_hwnd as *mut _),
                                PCWSTR(text_utf16.as_ptr()),
                            );
                        }
                        log::info!("Key capture started");
                    }
                } else if id == 20 && code == 0 {
                    // Remove blocked key button
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
                            )
                            .0 as i32;

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
                        let label_text_utf16: Vec<u16> = label_text
                            .encode_utf16()
                            .chain(std::iter::once(0))
                            .collect();
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
                        let label_text_utf16: Vec<u16> = label_text
                            .encode_utf16()
                            .chain(std::iter::once(0))
                            .collect();
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
                log::info!(
                    "WM_KEY_CAPTURED: vk_code=0x{:02X} ({}), key_name={}",
                    vk_code,
                    vk_code,
                    key_name
                );

                // Update button text to show captured key
                let btn_hwnd = *ADD_BUTTON_HWND.lock().unwrap();
                if btn_hwnd != 0 {
                    let btn_text = format!("\u{786E}\u{8BA4}\u{6DFB}\u{52A0} {}", key_name);
                    let text_utf16: Vec<u16> =
                        btn_text.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = SetWindowTextW(HWND(btn_hwnd as *mut _), PCWSTR(text_utf16.as_ptr()));
                }

                LRESULT::default()
            }
            WM_NOTIFY => {
                if let Some(result) = handle_max_sources_spin_delta(lparam) {
                    result
                } else if let Some(result) = handle_debounce_spin_delta(lparam) {
                    result
                } else {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }
            WM_TIMER => {
                if wparam.0 == 200 {
                    let peak = crate::audio::get_peak_sources();
                    let text = format!("\u{5CF0}\u{503C}\u{540C}\u{65F6}\u{64AD}\u{653E}\u{6570}: {}", peak);
                    let text_utf16: Vec<u16> =
                        text.encode_utf16().chain(std::iter::once(0)).collect();
                    let label_hwnd = *PEAK_LABEL_HWND.lock().unwrap();
                    if label_hwnd != 0 {
                        let _ = SetWindowTextW(
                            HWND(label_hwnd as *mut _),
                            PCWSTR(text_utf16.as_ptr()),
                        );
                    }
                }
                LRESULT::default()
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
                let _ = KillTimer(Some(hwnd), 200usize);
                *SETTINGS_HWND.lock().unwrap() = 0;
                *COMBOBOX_HWND.lock().unwrap() = 0;
                *VOLUME_SLIDER_HWND.lock().unwrap() = 0;
                *VOLUME_LABEL_HWND.lock().unwrap() = 0;
                *PITCH_SLIDER_HWND.lock().unwrap() = 0;
                *PITCH_LABEL_HWND.lock().unwrap() = 0;
                *MAX_SOURCES_COMBO_HWND.lock().unwrap() = 0;
                *MAX_SOURCES_EDITING.lock().unwrap() = false;
                *BLOCKED_KEYS_LISTVIEW_HWND.lock().unwrap() = 0;
                *PEAK_LABEL_HWND.lock().unwrap() = 0;
                *DEBOUNCE_EDIT_HWND.lock().unwrap() = 0;
                *DEBOUNCE_EDITING.lock().unwrap() = false;
                destroy_ui_fonts();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updown_message_constants_match_windows_sdk_values() {
        assert_eq!(UDM_SETBUDDY, 0x0469);
        assert_eq!(UDM_SETRANGE32, 0x046F);
        assert_eq!(UDM_SETPOS32, 0x0471);
        assert_eq!(UDN_DELTAPOS, 0xFFFF_FD2E);
    }

    #[test]
    fn max_sources_clamps_to_supported_ui_range() {
        assert_eq!(clamp_max_sources(1), 2);
        assert_eq!(clamp_max_sources(2), 2);
        assert_eq!(clamp_max_sources(12), 12);
        assert_eq!(clamp_max_sources(20), 20);
        assert_eq!(clamp_max_sources(21), 20);
    }
}
