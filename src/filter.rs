use std::sync::Mutex;

static CACHED_PROCESS: Mutex<Option<CachedProcess>> = Mutex::new(None);
static FOREGROUND_HOOK: Mutex<Option<isize>> = Mutex::new(None);
static mut HOOK_HWND: isize = 0;

struct CachedProcess {
    name: String,
    hwnd: isize,
}

pub fn get_foreground_process_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == 0 {
            return None;
        }

        {
            let cache = CACHED_PROCESS.lock().unwrap();
            if let Some(ref c) = *cache {
                if c.hwnd == hwnd {
                    return Some(c.name.clone());
                }
            }
        }

        let mut pid: u32 = 0;
        let tid = GetWindowThreadProcessId(hwnd, &mut pid);
        if tid == 0 || pid == 0 {
            return None;
        }

        let process = open_process_with_fallback(pid);
        if process == 0 {
            log::warn!("Cannot open process {} (protected system process?), unmuted", pid);
            *CACHED_PROCESS.lock().unwrap() = Some(CachedProcess {
                name: String::new(),
                hwnd,
            });
            return None;
        }

        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let result = QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(process);

        if result != 0 && size > 0 {
            let len = size as usize;
            if let Ok(name) = String::from_utf16(&buf[..len]) {
                let path = std::path::Path::new(&name);
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&name)
                    .to_lowercase();

                *CACHED_PROCESS.lock().unwrap() = Some(CachedProcess {
                    name: file_name.clone(),
                    hwnd,
                });

                return Some(file_name);
            }
        }

        log::warn!("QueryFullProcessImageNameW failed for PID {}, unmuted", pid);
        *CACHED_PROCESS.lock().unwrap() = Some(CachedProcess {
            name: String::new(),
            hwnd,
        });
        None
    }
}

unsafe fn open_process_with_fallback(pid: u32) -> isize {
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if process != 0 {
        return process;
    }
    OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid)
}

pub fn invalidate_cache() {
    *CACHED_PROCESS.lock().unwrap() = None;
}

pub fn install_foreground_hook(hwnd: isize) -> bool {
    unsafe {
        HOOK_HWND = hwnd;
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            0,
            Some(foreground_event_proc),
            0, 0,
            0,
        );
        if hook != 0 {
            *FOREGROUND_HOOK.lock().unwrap() = Some(hook);
            log::info!("Foreground event hook installed");
            true
        } else {
            log::error!("Failed to install foreground event hook");
            false
        }
    }
}

pub fn uninstall_foreground_hook() {
    unsafe {
        let hook = FOREGROUND_HOOK.lock().unwrap().take();
        if let Some(h) = hook {
            UnhookWinEvent(h);
            log::info!("Foreground event hook uninstalled");
        }
    }
}

pub fn should_mute(app_name: &str, list: &[String], mode: &str) -> bool {
    if list.is_empty() {
        return false;
    }
    let in_list = list.iter().any(|s| s == app_name);
    match mode {
        "BlackList" => in_list,
        "WhiteList" => !in_list,
        _ => false,
    }
}

pub fn check_and_apply_mute() {
    let app = get_foreground_process_name();
    let cfg = crate::config::get_config();
    match (app, cfg) {
        (Some(ref name), Some(ref cfg)) => {
            let mode = if cfg.filter_mode == crate::config::FilterMode::BlackList {
                "BlackList"
            } else {
                "WhiteList"
            };
            let muted = should_mute(name, &cfg.filter_list, mode);
            crate::audio::set_mute(muted);
            log::info!("App: {} filter={} list_len={} muted={}", name, mode, cfg.filter_list.len(), muted);
        }
        (None, _) => {
            crate::audio::set_mute(false);
            log::info!("No foreground app detected, unmuted");
        }
        _ => {}
    }
}

unsafe extern "system" fn foreground_event_proc(
    _hhook: isize,
    _event: u32,
    _hwnd: isize,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    invalidate_cache();
    let target = HOOK_HWND;
    if target != 0 {
        PostMessageW(target, crate::consts::WM_FOREGROUND_CHANGE, 0, 0);
    }
}

#[link(name = "user32")]
extern "system" {
    fn GetForegroundWindow() -> isize;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn SetWinEventHook(eventMin: u32, eventMax: u32, hmod: isize, lpfn: Option<unsafe extern "system" fn(isize, u32, isize, i32, i32, u32, u32)>, idProcess: u32, idThread: u32, dwFlags: u32) -> isize;
    fn UnhookWinEvent(hWinEventHook: isize) -> i32;
    fn PostMessageW(hWnd: isize, Msg: u32, wParam: isize, lParam: isize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
    fn CloseHandle(hObject: isize) -> i32;
    fn QueryFullProcessImageNameW(hProcess: isize, dwFlags: u32, lpExeName: *mut u16, lpdwSize: *mut u32) -> i32;
}

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
