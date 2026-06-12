use std::sync::Mutex;

const IDM_SHOW_SETTINGS: usize = 1001;
const IDM_TOGGLE_MUTE: usize = 1002;
const IDM_EXIT: usize = 1003;

static TRAY_HWND: Mutex<isize> = Mutex::new(0);
static TRAY_HICON: Mutex<isize> = Mutex::new(0);
static TRAY_HICON_MUTED: Mutex<isize> = Mutex::new(0);

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
        let icon_muted = *TRAY_HICON_MUTED.lock().unwrap();
        if icon_muted != 0 {
            DestroyIcon(icon_muted);
        }
        *TRAY_HWND.lock().unwrap() = 0;
        *TRAY_HICON.lock().unwrap() = 0;
        *TRAY_HICON_MUTED.lock().unwrap() = 0;
        log::info!("Tray icon removed");
    }
}

fn create_grayscale_icon(color_icon: isize) -> isize {
    unsafe {
        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(color_icon, &mut icon_info) == 0 {
            return color_icon;
        }

        let hdc = GetDC(0);
        let hdc_mem = CreateCompatibleDC(hdc);

        let old_bitmap = SelectObject(hdc_mem, icon_info.hbmColor);

        let mut bmi: BITMAPINFOHEADER = std::mem::zeroed();
        bmi.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.biWidth = 16;
        bmi.biHeight = -16;
        bmi.biPlanes = 1;
        bmi.biBitCount = 32;
        bmi.biCompression = 0;

        let mut pixels = vec![0u32; 256];
        GetDIBits(hdc_mem, icon_info.hbmColor, 0, 16, pixels.as_mut_ptr() as *mut u8, &mut bmi, 0);

        for pixel in pixels.iter_mut() {
            let b = (*pixel & 0xFF) as u8;
            let g = ((*pixel >> 8) & 0xFF) as u8;
            let r = ((*pixel >> 16) & 0xFF) as u8;
            let gray = (0.114 * b as f32 + 0.587 * g as f32 + 0.299 * r as f32) as u8;
            *pixel = 0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | gray as u32;
        }

        let mut ppv_bits: *mut u8 = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(hdc, &bmi, 0, &mut ppv_bits, 0, 0);

        let mut new_icon_info: ICONINFO = std::mem::zeroed();
        new_icon_info.fIcon = 1;
        new_icon_info.xHotspot = 0;
        new_icon_info.yHotspot = 0;
        new_icon_info.hbmColor = hbitmap;
        new_icon_info.hbmMask = icon_info.hbmMask;

        let gray_icon = CreateIconIndirect(&mut new_icon_info);

        SelectObject(hdc_mem, old_bitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(0, hdc);
        DeleteObject(hbitmap);
        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);

        gray_icon
    }
}

pub fn set_tray_icon_muted(muted: bool) {
    unsafe {
        let hwnd = *TRAY_HWND.lock().unwrap();
        if hwnd == 0 {
            return;
        }

        let icon = if muted {
            let mut icon_muted = *TRAY_HICON_MUTED.lock().unwrap();
            if icon_muted == 0 {
                let color_icon = *TRAY_HICON.lock().unwrap();
                icon_muted = create_grayscale_icon(color_icon);
                *TRAY_HICON_MUTED.lock().unwrap() = icon_muted;
            }
            icon_muted
        } else {
            *TRAY_HICON.lock().unwrap()
        };

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON;
        nid.hIcon = icon;

        Shell_NotifyIconW(NIM_MODIFY, &nid);
        log::info!("Tray icon updated: muted={}", muted);
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
const NIM_MODIFY: u32 = 1;
const NIF_MESSAGE: u32 = 1;
const NIF_ICON: u32 = 2;
const NIF_TIP: u32 = 4;
const IMAGE_ICON: u32 = 1;
const LR_DEFAULTCOLOR: u32 = 0;

const MF_STRING: u32 = 0;
const MF_SEPARATOR: u32 = 0x0800;
const TPM_RIGHTBUTTON: u32 = 2;
const TPM_BOTTOMALIGN: u32 = 0x0020;

#[repr(C)]
#[allow(non_snake_case)]
struct ICONINFO {
    fIcon: i32,
    xHotspot: u32,
    yHotspot: u32,
    hbmColor: isize,
    hbmMask: isize,
}

#[repr(C)]
#[allow(non_snake_case)]
struct BITMAPINFOHEADER {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

type HICON = isize;
type HDC = isize;
type HGDIOBJ = isize;
type HBITMAP = isize;

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
    fn GetIconInfo(hIcon: HICON, piconinfo: *mut ICONINFO) -> i32;
    fn CreateIconIndirect(piconinfo: *mut ICONINFO) -> isize;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetDC(hWnd: isize) -> HDC;
    fn ReleaseDC(hWnd: isize, hDC: HDC) -> i32;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn DeleteDC(hdc: HDC) -> i32;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn GetDIBits(hdc: HDC, hbm: HBITMAP, start: u32, cLines: u32, lpvBits: *mut u8, lpbmi: *mut BITMAPINFOHEADER, usage: u32) -> i32;
    fn CreateDIBSection(hdc: HDC, lpbmi: *const BITMAPINFOHEADER, usage: u32, ppvBits: *mut *mut u8, hSection: isize, offset: u32) -> HBITMAP;
    fn DeleteObject(hObject: HGDIOBJ) -> i32;
}
