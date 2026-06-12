use std::sync::Mutex;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

static SETTINGS_HWND: Mutex<isize> = Mutex::new(0);

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

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT::default()
        }
        WM_DESTROY => {
            *SETTINGS_HWND.lock().unwrap() = 0;
            LRESULT::default()
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
