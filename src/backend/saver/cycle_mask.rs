use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    RegisterClassW, CreateWindowExW, ShowWindow, WS_POPUP, SW_SHOW,
    WS_EX_TOPMOST, PeekMessageW, TranslateMessage, DispatchMessageW, MSG, DestroyWindow, WNDCLASSW,
    CS_HREDRAW, CS_VREDRAW
};
use windows_sys::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH};
use super::SystemMetrics;

unsafe extern "system" fn mask_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub struct CycleMask {
    hwnd: HWND,
}

impl CycleMask {
    pub fn new() -> Option<Self> {
        let class_name: Vec<u16> = "trance_mask_class\0".encode_utf16().collect();

        unsafe {
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(mask_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: std::ptr::null_mut(),
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: GetStockObject(BLACK_BRUSH) as HBRUSH,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            RegisterClassW(&wnd_class);

            let metrics = SystemMetrics::query();
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST,
                class_name.as_ptr(),
                std::ptr::null(),
                WS_POPUP,
                0,
                0,
                metrics.screen_w,
                metrics.screen_h,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );

            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_SHOW);

                let mut msg: MSG = std::mem::zeroed();
                while PeekMessageW(&mut msg, hwnd, 0, 0, 1) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                Some(CycleMask { hwnd })
            } else {
                None
            }
        }
    }
}

impl Drop for CycleMask {
    fn drop(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                DestroyWindow(self.hwnd);
                let mut msg: MSG = std::mem::zeroed();
                while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, 1) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
}
