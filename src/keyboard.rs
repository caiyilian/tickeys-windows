#![allow(non_snake_case, static_mut_refs)]
use crate::consts::WM_KEYDOWN_HOOK;
use std::time::SystemTime;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut HOOK: Option<HHOOK> = None;
static mut MAIN_HWND: Option<HWND> = None;

static mut LAST_KEY: i16 = -1;
static mut LAST_TIME: u64 = 0;

#[repr(C)]
struct KBDLLHOOKSTRUCT {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: usize,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn install(main_hwnd: HWND) -> Result<(), String> {
    unsafe {
        MAIN_HWND = Some(main_hwnd);

        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            None,
            0,
        );

        match hook {
            Ok(h) => {
                HOOK = Some(h);
                log::info!("Keyboard hook installed");
                Ok(())
            }
            Err(e) => Err(format!("Failed to install keyboard hook: {e:?}")),
        }
    }
}

pub fn uninstall() {
    unsafe {
        if let Some(hook) = HOOK.take() {
            let _ = UnhookWindowsHookEx(hook);
            log::info!("Keyboard hook uninstalled");
        }
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code == HC_ACTION as i32 {
        if w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN {
            let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kb.vkCode as u16;

            if !is_too_frequent(vk_code) {
                if let Some(hwnd) = MAIN_HWND {
                    let _ = PostMessageW(Some(hwnd), WM_KEYDOWN_HOOK, WPARAM(vk_code as usize), LPARAM::default());
                }
            }
        }
    }

    CallNextHookEx(None, n_code, w_param, l_param)
}

fn is_too_frequent(keycode: u16) -> bool {
    unsafe {
        let now = now_ms();
        let delta = now.wrapping_sub(LAST_TIME);
        if delta < 120 && LAST_KEY == keycode as i16 {
            LAST_TIME = now;
            return true;
        }
        LAST_KEY = keycode as i16;
        LAST_TIME = now;
        false
    }
}
