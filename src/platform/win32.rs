use std::{mem::size_of, ptr::addr_of};

use windows::{Win32::{UI::WindowsAndMessaging::{WNDCLASSEXW, WNDCLASS_STYLES, CS_DBLCLKS, CS_NOCLOSE, WNDPROC, HICON, HCURSOR, RegisterClassExW, CreateWindowExW, WINDOW_EX_STYLE, WINDOW_STYLE, HMENU}, Foundation::{HINSTANCE, HWND}, System::LibraryLoader::GetModuleHandleW, Graphics::Gdi::HBRUSH}, core::PCWSTR};

pub(crate) struct Window {

}

fn get_instance() -> Option<HINSTANCE> {
    unsafe { GetModuleHandleW(None).ok() }
}

struct WndClassId(u16);

fn register_class(
    menu_name: &str, 
    class_name: &str, 
    Some(wnd_proc): WNDPROC,
    icon: Option<HICON>,
    icon_small: Option<HICON>,
    cursor: Option<HCURSOR>, 
    background: Option<HBRUSH>, 
    no_close: bool
) -> Result<WndClassId, ()> {
    let close = if no_close { CS_NOCLOSE.0 } else { 0 };
    let menu_name_w = menu_name.encode_utf16().collect::<Vec<_>>();
    let class_name_w = class_name.encode_utf16().collect::<Vec<_>>();
    let wndclass = WNDCLASSEXW { 
        cbSize: size_of::<WNDCLASSEXW>(),
        style: WNDCLASS_STYLES(CS_DBLCLKS | close),
        lpfnWndProc: Some(wnd_proc), 
        cbClsExtra: 0, 
        cbWndExtra: 0, 
        hInstance: get_instance().unwrap(), 
        hIcon: icon.unwrap_or(0), 
        hCursor: cursor.unwrap_or(0), 
        hbrBackground: background.unwrap_or(0), 
        lpszMenuName: windows::core::PCWSTR(menu_name_w.as_ptr()), 
        lpszClassName: windows::core::PCWSTR(class_name_w.as_ptr()),
        hIconSm: icon_small.unwrap_or(0),
    };

    let res = unsafe { RegisterClassExW(addr_of!(wndclass)) };
    if res == 0 {
        Err(())
    } else {
        Ok(WndClassId(res))
    }
}

fn create_window(
    class_name: &str, 
    window_name: &str,
    ex_style: Option<WINDOW_EX_STYLE>, 
    style: Option<WINDOW_STYLE>, 
    x: i32, 
    y: i32, 
    width: i32, 
    height: i32,
    parent: Option<HWND>,
    menu: Option<HMENU>,
    instance: Option<HINSTANCE>
) -> Result<HWND, ()> {
    let class_name_w = class_name.encode_utf16().collect::<Vec<_>>();
    let window_name_w = window_name.encode_utf16().collect::<Vec<_>>();

    let res = unsafe {
         CreateWindowExW(
            style.unwrap_or(0), 
            PCWSTR(class_name_w.as_ptr()), 
            PCWSTR(window_name_w.as_ptr()), 
            style.unwrap_or(0), 
            x, 
            y, 
            width, 
            height, 
            parent.unwrap_or(0), 
            menu.unwrap_or(0), 
            instance.unwrap_or(0), 
            None,
         )
    };
    if res.0 == 0 {
        Err(())
    } else {
        Ok(res)
    }
}

impl Window {
    pub fn new() -> Self {
        
    }
}