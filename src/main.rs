

#[allow(dead_code)]
mod audio;
#[allow(dead_code)]
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
#[allow(dead_code)]
mod schemes;
#[allow(dead_code)]
mod tray;

use consts::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() {
    logging::init();

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
            log::info!("Key pressed: VK={vk_code} (0x{vk_code:02X})");
            LRESULT::default()
        }
        WM_DESTROY => {
            keyboard::uninstall();
            PostQuitMessage(0);
            LRESULT::default()
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
