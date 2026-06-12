use std::sync::Mutex;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

const TBS_HORZ: u32 = 0;
const TBS_AUTOTICKS: u32 = 0x0008;
const TBM_SETRANGE: u32 = 0x0406;
const TBM_SETPOS: u32 = 0x0407;
const TBM_GETPOS: u32 = 0x0400;

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

static SETTINGS_HWND: Mutex<isize> = Mutex::new(0);
static COMBOBOX_HWND: Mutex<isize> = Mutex::new(0);
static VOLUME_SLIDER_HWND: Mutex<isize> = Mutex::new(0);
static VOLUME_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static PITCH_SLIDER_HWND: Mutex<isize> = Mutex::new(0);
static PITCH_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static MAX_SOURCES_COMBO_HWND: Mutex<isize> = Mutex::new(0);
static FILTER_LISTVIEW_HWND: Mutex<isize> = Mutex::new(0);
static BLACKLIST_RADIO_HWND: Mutex<isize> = Mutex::new(0);
static WHITELIST_RADIO_HWND: Mutex<isize> = Mutex::new(0);

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
                center_window(hwnd.0 as isize);
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

fn create_controls(hwnd: isize) {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap();

        // Scheme selector ComboBox
        let combo_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("COMBOBOX"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
            20,
            20,
            360,
            200,
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

            for (index, scheme) in schemes.iter().enumerate() {
                let display_name: Vec<u16> = scheme.display_name.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = SendMessageW(
                    combo_hwnd,
                    CB_ADDSTRING,
                    Some(WPARAM(0)),
                    Some(LPARAM(display_name.as_ptr() as isize)),
                );

                if scheme.name == current_scheme {
                    let _ = SendMessageW(
                        combo_hwnd,
                        CB_SETCURSEL,
                        Some(WPARAM(index)),
                        Some(LPARAM(0)),
                    );
                }
            }

            log::info!("ComboBox created with {} schemes", schemes.len());
        }

        // Volume label
        let current_volume = crate::config::get_config()
            .map(|c| c.volume)
            .unwrap_or(0.5);
        let initial_percentage = (current_volume * 100.0) as i32;
        let initial_label_text = format!("\u{97F3}\u{91CF}: {}%", initial_percentage);
        let initial_label_text_utf16: Vec<u16> = initial_label_text.encode_utf16().chain(std::iter::once(0)).collect();

        let volume_label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(initial_label_text_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            20,
            60,
            100,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(2 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(volume_label_hwnd) = volume_label_hwnd {
            *VOLUME_LABEL_HWND.lock().unwrap() = volume_label_hwnd.0 as isize;
        }

        // Volume Trackbar
        let volume_slider_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_trackbar32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
            120,
            60,
            260,
            30,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(3 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(volume_slider_hwnd) = volume_slider_hwnd {
            *VOLUME_SLIDER_HWND.lock().unwrap() = volume_slider_hwnd.0 as isize;

            let current_volume = crate::config::get_config()
                .map(|c| c.volume)
                .unwrap_or(0.5);

            let slider_pos = (current_volume * 100.0) as isize;

            let _ = SendMessageW(
                volume_slider_hwnd,
                TBM_SETRANGE,
                Some(WPARAM(1)),
                Some(LPARAM(((0 << 16) | 500) as isize)),
            );

            let _ = SendMessageW(
                volume_slider_hwnd,
                TBM_SETPOS,
                Some(WPARAM(1)),
                Some(LPARAM(slider_pos)),
            );

            let percentage = (current_volume * 100.0) as i32;
            let label_text = format!("\u{97F3}\u{91CF}: {}%", percentage);
            let label_hwnd = *VOLUME_LABEL_HWND.lock().unwrap();
            if label_hwnd != 0 {
                let label_text_utf16: Vec<u16> = label_text.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = SetWindowTextW(
                    HWND(label_hwnd as *mut _),
                    PCWSTR(label_text_utf16.as_ptr()),
                );
            }

            log::info!("Volume slider created with {}%", percentage);
        }

        // Pitch label
        let current_pitch = crate::config::get_config()
            .map(|c| c.pitch)
            .unwrap_or(1.0);
        let initial_pitch_display = format!("\u{97F3}\u{8C03}: {:.1}", current_pitch);
        let initial_pitch_text_utf16: Vec<u16> = initial_pitch_display.encode_utf16().chain(std::iter::once(0)).collect();

        let pitch_label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(initial_pitch_text_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            20,
            100,
            100,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(4 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(pitch_label_hwnd) = pitch_label_hwnd {
            *PITCH_LABEL_HWND.lock().unwrap() = pitch_label_hwnd.0 as isize;
        }

        // Pitch Trackbar
        let pitch_slider_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_trackbar32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
            120,
            100,
            260,
            30,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(5 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(pitch_slider_hwnd) = pitch_slider_hwnd {
            *PITCH_SLIDER_HWND.lock().unwrap() = pitch_slider_hwnd.0 as isize;

            let slider_pos = (current_pitch * 100.0) as isize;

            let _ = SendMessageW(
                pitch_slider_hwnd,
                TBM_SETRANGE,
                Some(WPARAM(1)),
                Some(LPARAM(((50 << 16) | 200) as isize)),
            );

            let _ = SendMessageW(
                pitch_slider_hwnd,
                TBM_SETPOS,
                Some(WPARAM(1)),
                Some(LPARAM(slider_pos)),
            );

            log::info!("Pitch slider created with {:.1}", current_pitch);
        }

        // Max sources label
        let max_sources_label_text = "\u{540C}\u{65F6}\u{64AD}\u{653E}\u{6570}(\u{8D8A}\u{5927}\u{8D8A}\u{4E0D}\u{5BB9}\u{6613}\u{622A}\u{65AD})";
        let max_sources_label_utf16: Vec<u16> = max_sources_label_text.encode_utf16().chain(std::iter::once(0)).collect();

        let _max_sources_label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(max_sources_label_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            20,
            140,
            100,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(6 as *mut _)),
            Some(instance.into()),
            None,
        );

        // Max sources ComboBox
        let max_sources_combo_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("COMBOBOX"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
            120,
            140,
            260,
            200,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(7 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(max_sources_combo_hwnd) = max_sources_combo_hwnd {
            *MAX_SOURCES_COMBO_HWND.lock().unwrap() = max_sources_combo_hwnd.0 as isize;

            let current_max_sources = crate::config::get_config()
                .map(|c| c.max_sources)
                .unwrap_or(2);

            for i in 1..=6 {
                let option_text = format!("{}", i);
                let option_utf16: Vec<u16> = option_text.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = SendMessageW(
                    max_sources_combo_hwnd,
                    CB_ADDSTRING,
                    Some(WPARAM(0)),
                    Some(LPARAM(option_utf16.as_ptr() as isize)),
                );

                if i == current_max_sources {
                    let _ = SendMessageW(
                        max_sources_combo_hwnd,
                        CB_SETCURSEL,
                        Some(WPARAM((i - 1) as usize)),
                        Some(LPARAM(0)),
                    );
                }
            }

            log::info!("Max sources ComboBox created with current value {}", current_max_sources);
        }

        // Filter section - ListView
        let filter_label_text = "\u{9ED1}\u{767D}\u{540D}\u{5355}";
        let filter_label_utf16: Vec<u16> = filter_label_text.encode_utf16().chain(std::iter::once(0)).collect();

        let _filter_label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(filter_label_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE,
            20,
            180,
            100,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(8 as *mut _)),
            Some(instance.into()),
            None,
        );

        // Get current config for filter list
        let current_config = crate::config::get_config();
        let filter_list = current_config.as_ref().map(|c| &c.filter_list).cloned().unwrap_or_default();
        let filter_mode = current_config.as_ref().map(|c| c.filter_mode.clone()).unwrap_or(crate::config::FilterMode::BlackList);

        // ListView for filter list
        let filter_listview_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("SysListView32"),
            None,
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0001 | 0x0002), // LVS_REPORT | LVS_SHOWSELALWAYS
            20,
            200,
            360,
            100,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(9 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(filter_listview_hwnd) = filter_listview_hwnd {
            *FILTER_LISTVIEW_HWND.lock().unwrap() = filter_listview_hwnd.0 as isize;

            // Add column
            let column_text = "\u{5E94}\u{7528}\u{540D}\u{79F0}";
            let column_text_utf16: Vec<u16> = column_text.encode_utf16().chain(std::iter::once(0)).collect();
            let mut column = LVCOLUMNW {
                mask: 0x0001, // LVCF_TEXT
                pszText: column_text_utf16.as_ptr(),
                cx: 340,
                ..std::mem::zeroed()
            };
            let _ = SendMessageW(
                filter_listview_hwnd,
                0x1006, // LVM_INSERTCOLUMNW
                Some(WPARAM(0)),
                Some(LPARAM(&mut column as *mut _ as isize)),
            );

            // Load items into ListView
            for (index, app_name) in filter_list.iter().enumerate() {
                let app_name_utf16: Vec<u16> = app_name.encode_utf16().chain(std::iter::once(0)).collect();
                let mut item = LVCITEMW {
                    mask: 0x0001, // LVCF_TEXT
                    iItem: index as i32,
                    pszText: app_name_utf16.as_ptr() as *mut _,
                    ..std::mem::zeroed()
                };
                let _ = SendMessageW(
                    filter_listview_hwnd,
                    0x1007, // LVM_INSERTITEMW
                    Some(WPARAM(0)),
                    Some(LPARAM(&mut item as *mut _ as isize)),
                );
            }

            log::info!("Filter ListView created with {} items", filter_list.len());
        }

        // Radio buttons for filter mode
        let blacklist_radio_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{9ED1}\u{540D}\u{5355}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0009), // BS_AUTORADIOBUTTON
            20,
            310,
            80,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(10 as *mut _)),
            Some(instance.into()),
            None,
        );

        let whitelist_radio_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{767D}\u{540D}\u{5355}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0009), // BS_AUTORADIOBUTTON
            110,
            310,
            80,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(11 as *mut _)),
            Some(instance.into()),
            None,
        );

        if let Ok(blacklist_radio_hwnd) = blacklist_radio_hwnd {
            *BLACKLIST_RADIO_HWND.lock().unwrap() = blacklist_radio_hwnd.0 as isize;
            if filter_mode == crate::config::FilterMode::BlackList {
                let _ = SendMessageW(
                    blacklist_radio_hwnd,
                    0x00F1, // BM_SETCHECK
                    Some(WPARAM(1)), // BST_CHECKED
                    Some(LPARAM(0)),
                );
            }
        }

        if let Ok(whitelist_radio_hwnd) = whitelist_radio_hwnd {
            *WHITELIST_RADIO_HWND.lock().unwrap() = whitelist_radio_hwnd.0 as isize;
            if filter_mode == crate::config::FilterMode::WhiteList {
                let _ = SendMessageW(
                    whitelist_radio_hwnd,
                    0x00F1, // BM_SETCHECK
                    Some(WPARAM(1)), // BST_CHECKED
                    Some(LPARAM(0)),
                );
            }
        }

        // Add/Remove buttons
        let _add_button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{6DFB}\u{52A0}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001), // BS_PUSHBUTTON
            300,
            310,
            40,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(12 as *mut _)),
            Some(instance.into()),
            None,
        );

        let _remove_button_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("\u{79FB}\u{9664}"),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000001), // BS_PUSHBUTTON
            340,
            310,
            40,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(13 as *mut _)),
            Some(instance.into()),
            None,
        );

        // Adjust window size to fit new controls
        let _ = SetWindowPos(
            HWND(hwnd as *mut _),
            None,
            0,
            0,
            400,
            400,
            SWP_NOMOVE | SWP_NOZORDER,
        );

        // Version label at bottom
        let version_text = format!("v{}", crate::consts::CURRENT_VERSION);
        let version_utf16: Vec<u16> = version_text.encode_utf16().chain(std::iter::once(0)).collect();

        let _version_label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(version_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000010), // SS_CENTER | SS_NOTIFY
            20,
            380,
            360,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(14 as *mut _)),
            Some(instance.into()),
            None,
        );

        // Website link
        let website_text = "\u{8BBF}\u{95EE}\u{5B98}\u{7F51}";
        let website_utf16: Vec<u16> = website_text.encode_utf16().chain(std::iter::once(0)).collect();

        let _website_label_hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            PCWSTR(website_utf16.as_ptr()),
            WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x00000010 | 0x00000200), // SS_CENTER | SS_NOTIFY
            20,
            380,
            360,
            20,
            Some(HWND(hwnd as *mut _)),
            Some(HMENU(15 as *mut _)),
            Some(instance.into()),
            None,
        );
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let code = (wparam.0 >> 16) & 0xFFFF;
            let id = wparam.0 & 0xFFFF;

            if id == 1 && code == 0 { // CBN_SELCHANGE for scheme selector
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
            } else if id == 7 && code == 0 { // CBN_SELCHANGE for max sources
                let max_sources_combo = *MAX_SOURCES_COMBO_HWND.lock().unwrap();
                if max_sources_combo != 0 {
                    let index = SendMessageW(
                        HWND(max_sources_combo as *mut _),
                        CB_GETCURSEL,
                        Some(WPARAM(0)),
                        Some(LPARAM(0)),
                    );

                    if index.0 != -1 {
                        let new_max_sources = (index.0 + 1) as usize;
                        crate::audio::rebuild_player(new_max_sources);

                        if let Some(mut cfg) = crate::config::get_config() {
                            cfg.max_sources = new_max_sources;
                            crate::config::update_config(&cfg);
                        }

                        log::info!("Max sources changed to {}", new_max_sources);
                    }
                }
            } else if id == 10 && code == 0 { // BlackList radio button
                if let Some(mut cfg) = crate::config::get_config() {
                    cfg.filter_mode = crate::config::FilterMode::BlackList;
                    crate::config::update_config(&cfg);
                    log::info!("Filter mode changed to BlackList");
                }
            } else if id == 11 && code == 0 { // WhiteList radio button
                if let Some(mut cfg) = crate::config::get_config() {
                    cfg.filter_mode = crate::config::FilterMode::WhiteList;
                    crate::config::update_config(&cfg);
                    log::info!("Filter mode changed to WhiteList");
                }
            } else if id == 12 && code == 0 { // Add button
                // Open file dialog to add applications
                let hwnd = HWND(hwnd.0 as *mut _);
                let mut file_path = [0u16; 260];
                let filter_text = "Applications (*.exe)\0*.exe\0\0";
                let filter_utf16: Vec<u16> = filter_text.encode_utf16().chain(std::iter::once(0)).collect();
                let title_text = "\u{6DFB}\u{52A0}\u{5E94}\u{7528}";
                let title_utf16: Vec<u16> = title_text.encode_utf16().chain(std::iter::once(0)).collect();
                let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
                ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
                ofn.hwndOwner = hwnd;
                ofn.lpstrFilter = filter_utf16.as_ptr();
                ofn.lpstrFile = file_path.as_mut_ptr();
                ofn.nMaxFile = 260;
                ofn.lpstrTitle = title_utf16.as_ptr();
                ofn.Flags = 0x00080000 | 0x00001000 | 0x00000200; // OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_ALLOWMULTISELECT

                if unsafe { GetOpenFileNameW(&mut ofn) } != 0 {
                    // Parse selected files
                    let files: Vec<String> = file_path
                        .split(|&c| c == 0)
                        .filter(|s| !s.is_empty())
                        .skip(1) // Skip directory
                        .filter_map(|s| {
                            let path_str = String::from_utf16_lossy(s);
                            let path = std::path::Path::new(&path_str);
                            path.file_name().map(|n| n.to_string_lossy().to_string())
                        })
                        .collect();

                    if !files.is_empty() {
                        if let Some(mut cfg) = crate::config::get_config() {
                            for file in &files {
                                if !cfg.filter_list.contains(file) {
                                    cfg.filter_list.push(file.clone());
                                }
                            }
                            cfg.filter_list.sort();
                            cfg.filter_list.dedup();
                            crate::config::update_config(&cfg);

                            // Update ListView
                            let listview_hwnd = *FILTER_LISTVIEW_HWND.lock().unwrap();
                            if listview_hwnd != 0 {
                                for file in &files {
                                    let app_name_utf16: Vec<u16> = file.encode_utf16().chain(std::iter::once(0)).collect();
                                    let mut item = LVCITEMW {
                                        mask: 0x0001, // LVCF_TEXT
                                        iItem: cfg.filter_list.len() as i32 - 1,
                                        pszText: app_name_utf16.as_ptr() as *mut _,
                                        ..std::mem::zeroed()
                                    };
                                    let _ = SendMessageW(
                                        HWND(listview_hwnd as *mut _),
                                        0x1007, // LVM_INSERTITEMW
                                        Some(WPARAM(0)),
                                        Some(LPARAM(&mut item as *mut _ as isize)),
                                    );
                                }
                            }

                            log::info!("Added {} applications to filter list", files.len());
                        }
                    }
                }
            } else if id == 13 && code == 0 { // Remove button
                // Remove selected items from ListView
                let listview_hwnd = *FILTER_LISTVIEW_HWND.lock().unwrap();
                if listview_hwnd != 0 {
                    let mut selected_indices = Vec::new();
                    let mut index = -1;

                    // Get selected items (in reverse order to remove from end)
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
                            // Remove items in reverse order to maintain correct indices
                            for &idx in selected_indices.iter().rev() {
                                if (idx as usize) < cfg.filter_list.len() {
                                    cfg.filter_list.remove(idx as usize);
                                }
                            }
                            crate::config::update_config(&cfg);
                            log::info!("Removed {} applications from filter list", selected_indices.len());
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
                let pos = SendMessageW(
                    HWND(volume_slider as *mut _),
                    TBM_GETPOS,
                    Some(WPARAM(0)),
                    Some(LPARAM(0)),
                );

                let volume = pos.0 as f32 / 100.0;
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
                let pos = SendMessageW(
                    HWND(pitch_slider as *mut _),
                    TBM_GETPOS,
                    Some(WPARAM(0)),
                    Some(LPARAM(0)),
                );

                let pitch = pos.0 as f32 / 100.0;
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
        WM_NOTIFY => {
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
            *FILTER_LISTVIEW_HWND.lock().unwrap() = 0;
            *BLACKLIST_RADIO_HWND.lock().unwrap() = 0;
            *WHITELIST_RADIO_HWND.lock().unwrap() = 0;
            LRESULT::default()
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
}
