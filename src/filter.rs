use std::sync::Mutex;

static CACHED_PROCESS: Mutex<Option<CachedProcess>> = Mutex::new(None);

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

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process == 0 {
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

        None
    }
}

pub fn invalidate_cache() {
    *CACHED_PROCESS.lock().unwrap() = None;
}

extern "system" {
    fn GetForegroundWindow() -> isize;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
    fn CloseHandle(hObject: isize) -> i32;
    fn QueryFullProcessImageNameW(hProcess: isize, dwFlags: u32, lpExeName: *mut u16, lpdwSize: *mut u32) -> i32;
}

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
