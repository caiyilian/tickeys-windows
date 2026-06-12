use std::sync::Mutex;

const IDM_SHOW_SETTINGS: usize = 1001;
const IDM_TOGGLE_MUTE: usize = 1002;
const IDM_EXIT: usize = 1003;

static TRAY_HWND: Mutex<isize> = Mutex::new(0);
static TRAY_HICON: Mutex<isize> = Mutex::new(0);

pub fn install_tray(hwnd: isize, instance: isize) -> bool {
    unsafe {
        let icon = LoadImageW(
            instance,
            1 as *const u16 as isize,
            IMAGE_ICON,
            16, 16,
            LR_DEFAULTCOLOR,
        );
        if icon == 0 {
            log::error!("Failed to load tray icon from resources");
            return false;
        }

        let mut tip: [u16; 128] = [0; 128];
        let text = "Tickeys\0";
        for (i, c) in text.encode_utf16().enumerate() {
            if i < 127 {
                tip[i] = c;
            }
        }

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = crate::consts::WM_TRAYICON;
        nid.hIcon = icon;
        nid.szTip = tip;

        let result = Shell_NotifyIconW(NIM_ADD, &nid);
        if result == 0 {
            log::error!("Shell_NotifyIconW failed");
            return false;
        }

        *TRAY_HWND.lock().unwrap() = hwnd;
        *TRAY_HICON.lock().unwrap() = icon;

        log::info!("Tray icon installed");
        true
    }
}

pub fn remove_tray() {
    unsafe {
        let hwnd = *TRAY_HWND.lock().unwrap();
        if hwnd != 0 {
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            Shell_NotifyIconW(NIM_DELETE, &nid);
        }
        let icon = *TRAY_HICON.lock().unwrap();
        if icon != 0 {
            DestroyIcon(icon);
        }
        *TRAY_HWND.lock().unwrap() = 0;
        *TRAY_HICON.lock().unwrap() = 0;
        log::info!("Tray icon removed");
    }
}

pub fn show_context_menu(hwnd: isize) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu == 0 {
            return;
        }

        AppendMenuW(menu, MF_STRING, IDM_SHOW_SETTINGS, make_wstr("\u{663E}\u{793A}\u{8BBE}\u{7F6E}\0"));
        AppendMenuW(menu, MF_STRING, IDM_TOGGLE_MUTE, make_wstr("\u{542F}\u{7528}/\u{7981}\u{7528}\0"));
        AppendMenuW(menu, MF_SEPARATOR, 0, 0 as isize);
        AppendMenuW(menu, MF_STRING, IDM_EXIT, make_wstr("\u{9000}\u{51FA}\0"));

        let mut p = POINT { x: 0, y: 0 };
        GetCursorPos(&mut p);

        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            p.x,
            p.y,
            0,
            hwnd,
            0 as isize,
        );
        DestroyMenu(menu);
    }
}

fn make_wstr(s: &str) -> isize {
    let buf: Vec<u16> = s.encode_utf16().collect();
    let ptr = buf.as_ptr() as isize;
    std::mem::forget(buf);
    ptr
}

#[repr(C)]
#[allow(non_snake_case)]
struct NOTIFYICONDATAW {
    cbSize: u32,
    hWnd: isize,
    uID: u32,
    uFlags: u32,
    uCallbackMessage: u32,
    hIcon: isize,
    szTip: [u16; 128],
    dwState: u32,
    dwStateMask: u32,
    szInfo: [u16; 256],
    uVersion: u32,
    szInfoTitle: [u16; 64],
    dwInfoFlags: u32,
    guidItem: [u8; 16],
    hBalloonIcon: isize,
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

const NIM_ADD: u32 = 0;
const NIM_DELETE: u32 = 2;
const NIF_MESSAGE: u32 = 1;
const NIF_ICON: u32 = 2;
const NIF_TIP: u32 = 4;
const IMAGE_ICON: u32 = 1;
const LR_DEFAULTCOLOR: u32 = 0;

const MF_STRING: u32 = 0;
const MF_SEPARATOR: u32 = 0x0800;
const TPM_RIGHTBUTTON: u32 = 2;
const TPM_BOTTOMALIGN: u32 = 0x0020;

#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn LoadImageW(hInst: isize, name: isize, typ: u32, cx: i32, cy: i32, fuLoad: u32) -> isize;
    fn DestroyIcon(hIcon: isize) -> i32;
    fn CreatePopupMenu() -> isize;
    fn AppendMenuW(hMenu: isize, uFlags: u32, uIDNewItem: usize, lpNewItem: isize) -> i32;
    fn TrackPopupMenu(hMenu: isize, uFlags: u32, x: i32, y: i32, nReserved: i32, hWnd: isize, prcRect: isize) -> i32;
    fn DestroyMenu(hMenu: isize) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn SetForegroundWindow(hWnd: isize) -> i32;
}
