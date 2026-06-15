#![allow(non_snake_case, static_mut_refs)]
use crate::consts::{WM_KEYDOWN_HOOK, WM_SHOW_SETTINGS, OPEN_SETTINGS_KEY_SEQ};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut HOOK: Option<HHOOK> = None;
static mut MAIN_HWND: Option<HWND> = None;

static mut LAST_KEY: i16 = -1;
static mut LAST_TIME: u64 = 0;

static mut KEY_HISTORY: Option<VecDeque<u32>> = None;

static DEBOUNCE_MS: AtomicU32 = AtomicU32::new(20);

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

pub fn set_debounce_ms(ms: u32) {
    DEBOUNCE_MS.store(ms.clamp(10, 500), Ordering::Relaxed);
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
                // Add to key history
                if KEY_HISTORY.is_none() {
                    KEY_HISTORY = Some(VecDeque::with_capacity(20));
                }
                if let Some(ref mut history) = KEY_HISTORY {
                    history.push_back(vk_code as u32);
                    if history.len() > 20 {
                        history.pop_front();
                    }
                }

                // Check for settings key sequence
                if check_settings_shortcut() {
                    // Send message to show settings window
                    if let Some(hwnd) = MAIN_HWND {
                        let _ = PostMessageW(Some(hwnd), WM_SHOW_SETTINGS, WPARAM(0), LPARAM(0));
                    }
                }

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
        if delta < DEBOUNCE_MS.load(Ordering::Relaxed) as u64 && LAST_KEY == keycode as i16 {
            LAST_TIME = now;
            return true;
        }
        LAST_KEY = keycode as i16;
        LAST_TIME = now;
        false
    }
}

fn check_settings_shortcut() -> bool {
    unsafe {
        if let Some(ref history) = KEY_HISTORY {
            for seq in OPEN_SETTINGS_KEY_SEQ {
                if history.len() >= seq.len() {
                    let tail: Vec<u32> = history.iter().rev().take(seq.len()).cloned().collect();
                    let seq_reversed: Vec<u32> = seq.iter().rev().cloned().collect();
                    if tail == seq_reversed {
                        return true;
                    }
                }
            }
        }
        false
    }
}
