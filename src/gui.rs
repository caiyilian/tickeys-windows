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

static SETTINGS_HWND: Mutex<isize> = Mutex::new(0);
static COMBOBOX_HWND: Mutex<isize> = Mutex::new(0);
static VOLUME_SLIDER_HWND: Mutex<isize> = Mutex::new(0);
static VOLUME_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static PITCH_SLIDER_HWND: Mutex<isize> = Mutex::new(0);
static PITCH_LABEL_HWND: Mutex<isize> = Mutex::new(0);
static MAX_SOURCES_COMBO_HWND: Mutex<isize> = Mutex::new(0);

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
            LRESULT::default()
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
