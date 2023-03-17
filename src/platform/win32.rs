#![allow(dead_code)]

use std::{mem::size_of, ptr::addr_of};

use windows::{Win32::{UI::WindowsAndMessaging::{WNDCLASSEXW, WNDCLASS_STYLES, CS_DBLCLKS, CS_NOCLOSE, HICON, HCURSOR, RegisterClassExW, CreateWindowExW, WINDOW_EX_STYLE, WINDOW_STYLE, HMENU, ShowWindow, SW_NORMAL, IDI_APPLICATION, LoadIconW, LoadCursorW, SW_HIDE, WNDPROC}, Foundation::{HINSTANCE, HWND, WPARAM, LPARAM, LRESULT, GetLastError, WIN32_ERROR}, System::LibraryLoader::GetModuleHandleW, Graphics::Gdi::{HBRUSH, COLOR_WINDOW, UpdateWindow}}, core::PCWSTR};

#[derive(Clone, Default)]
pub(crate) struct Window {
    hwnd: HWND,
    hinstance: Option<HINSTANCE>,
    visible: bool,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: Option<HWND>,
    icon: HICON,
    icon_small: HICON,
    menu: Option<HMENU>,
    menu_name: String,
    style: WINDOW_STYLE,
    style_ex: WINDOW_EX_STYLE,
    class_name: String,
    name: String,
    cursor: HCURSOR,
    background: HBRUSH,
    no_close: bool,
    wnd_proc: WNDPROC,
}

impl Window {
    pub(crate) fn new(class_name: &str, width: i32, height: i32, wnd_proc: WndProc) -> Self {
        Self {
            class_name: class_name.to_owned(),
            width,
            height,
            wnd_proc: Some(wnd_proc),
            ..Default::default()
        }
    }

    pub(crate) fn register(&self) -> Result<WndClassId, WIN32_ERROR> {
        register_class(
            &self.menu_name, 
            &self.class_name, 
            self.wnd_proc.unwrap(), 
            Some(self.icon), 
            Some(self.icon_small), 
            Some(self.cursor), 
            Some(self.background), 
            self.no_close
        )
    } 

    pub(crate) fn create(&mut self) -> Result<HWND, WIN32_ERROR> {
        let res = create_window(
            &self.class_name, 
            &self.name, 
            self.visible, 
            Some(self.style_ex), 
            Some(self.style), 
            self.x, 
            self.y, 
            self.width, 
            self.height, 
            self.parent, 
            self.menu, 
            self.hinstance
        );

        if let Ok(hwnd) = res {
            self.hwnd = hwnd;
        }

        res
    }
}

fn get_instance() -> Option<HINSTANCE> {
    unsafe { GetModuleHandleW(None).ok() }
}

pub(crate) struct WndClassId(u16);

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

fn register_class(
    menu_name: &str, 
    class_name: &str, 
    wnd_proc: WndProc,
    icon: Option<HICON>,
    icon_small: Option<HICON>,
    cursor: Option<HCURSOR>, 
    background: Option<HBRUSH>, 
    no_close: bool
) -> Result<WndClassId, WIN32_ERROR> {
    let close = if no_close { CS_NOCLOSE } else { WNDCLASS_STYLES(0) };
    let mut menu_name_w = menu_name.encode_utf16().collect::<Vec<_>>();
    menu_name_w.push(0x0000);
    let mut class_name_w = class_name.encode_utf16().collect::<Vec<_>>();
    class_name_w.push(0x0000);

    let wndclass = WNDCLASSEXW { 
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_DBLCLKS | close,
        lpfnWndProc: Some(wnd_proc), 
        cbClsExtra: 0, 
        cbWndExtra: 0, 
        hInstance: get_instance().unwrap(), 
        hIcon: icon.unwrap_or_else(|| unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap()), 
        hCursor: cursor.unwrap_or_else(|| unsafe { LoadCursorW(None, IDI_APPLICATION) }.unwrap()), 
        hbrBackground: background.unwrap_or(HBRUSH((COLOR_WINDOW.0 + 1) as _)),
        lpszMenuName: windows::core::PCWSTR(menu_name_w.as_ptr()), 
        lpszClassName: windows::core::PCWSTR(class_name_w.as_ptr()),
        hIconSm: icon_small.unwrap_or_else(|| unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap()),
    };

    let res = unsafe { RegisterClassExW(addr_of!(wndclass)) };
    if res == 0 {
        Err(unsafe { GetLastError() })
    } else {
        Ok(WndClassId(res))
    }
}

fn create_window(
    class_name: &str, 
    window_name: &str,
    visible: bool,
    style_ex: Option<WINDOW_EX_STYLE>, 
    style: Option<WINDOW_STYLE>, 
    x: i32, 
    y: i32, 
    width: i32, 
    height: i32,
    parent: Option<HWND>,
    menu: Option<HMENU>,
    instance: Option<HINSTANCE>
) -> Result<HWND, WIN32_ERROR> {
    let mut class_name_w = class_name.encode_utf16().collect::<Vec<_>>();
    class_name_w.push(0x0000);
    let mut window_name_w = window_name.encode_utf16().collect::<Vec<_>>();
    window_name_w.push(0x0000);

    let hwnd = unsafe {
         CreateWindowExW(
            style_ex.unwrap_or(WINDOW_EX_STYLE(0)), 
            PCWSTR(class_name_w.as_ptr()), 
            PCWSTR(window_name_w.as_ptr()), 
            style.unwrap_or(WINDOW_STYLE(0)), 
            x, 
            y, 
            width, 
            height, 
            parent.unwrap_or(HWND(0)), 
            menu.unwrap_or(HMENU(0)), 
            instance.unwrap_or(HINSTANCE(0)), 
            None,
         )
    };
    if hwnd.0 == 0 {
        Err(unsafe { GetLastError() })
    } else {
        let ncmdshow = if visible { SW_NORMAL } else { SW_HIDE };
        unsafe { ShowWindow(hwnd, ncmdshow) };
        unsafe { UpdateWindow(hwnd) };
        Ok(hwnd)
    }
}

mod tests {
    use windows::Win32::{Foundation::{LRESULT, HWND, LPARAM, WPARAM}, UI::WindowsAndMessaging::{WM_KEYDOWN, WM_CLOSE, DestroyWindow, WM_DESTROY, PostQuitMessage, DefWindowProcW}};

    #[allow(unused)]
    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_KEYDOWN => { DestroyWindow(hwnd); LRESULT(0) },
            WM_CLOSE => { DestroyWindow(hwnd); LRESULT(0) },
            WM_DESTROY => { 
                PostQuitMessage(0); 
                LRESULT(0) 
            },
            _ => { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    #[test]
    fn cw_test() {
        use crate::platform::win32::{create_window, register_class, get_instance};
        use std::{ptr::{addr_of_mut, addr_of}};
        use windows::Win32::UI::WindowsAndMessaging::{WS_OVERLAPPEDWINDOW, TranslateMessage, GetMessageW, MSG, DispatchMessageW};

        register_class(
            "test_menu", 
            "test_class", 
            wnd_proc, 
            None, 
            None, 
            None, 
            None, 
            false
        ).unwrap();

        let hwnd = create_window(
            "test_class",
            "test_window",
            true,
            None,
            Some(WS_OVERLAPPEDWINDOW),
            0,
            0,
            600,
            400,
            None,
            None,
            get_instance()
        ).unwrap();

        let mut msg = MSG::default();
        loop {
            if unsafe { GetMessageW(addr_of_mut!(msg), hwnd, 0, 0).0 <= 0 } {
                break;
            }

            unsafe { TranslateMessage(addr_of!(msg)) };
            unsafe { DispatchMessageW(addr_of!(msg)) };

            //if SHOULD_QUIT.load(std::sync::atomic::Ordering::Relaxed) {
            //    break;
            //}
        }
    }
}