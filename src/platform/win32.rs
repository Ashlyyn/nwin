use std::{mem::size_of, ptr::addr_of};

use windows::{Win32::{UI::WindowsAndMessaging::{WNDCLASSEXW, WNDCLASS_STYLES, CS_DBLCLKS, CS_NOCLOSE, HICON, HCURSOR, RegisterClassExW, CreateWindowExW, WINDOW_EX_STYLE, WINDOW_STYLE, HMENU}, Foundation::{HINSTANCE, HWND, WPARAM, LPARAM, LRESULT}, System::LibraryLoader::GetModuleHandleW, Graphics::Gdi::HBRUSH}, core::PCWSTR};

pub(crate) struct Window {

}

fn get_instance() -> Option<HINSTANCE> {
    unsafe { GetModuleHandleW(None).ok() }
}

struct WndClassId(u16);

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
) -> Result<WndClassId, ()> {
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
        hIcon: icon.unwrap_or(HICON(0)), 
        hCursor: cursor.unwrap_or(HCURSOR(0)), 
        hbrBackground: background.unwrap_or(HBRUSH(0)), 
        lpszMenuName: windows::core::PCWSTR(menu_name_w.as_ptr()), 
        lpszClassName: windows::core::PCWSTR(class_name_w.as_ptr()),
        hIconSm: icon_small.unwrap_or(HICON(0)),
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
    style_ex: Option<WINDOW_EX_STYLE>, 
    style: Option<WINDOW_STYLE>, 
    x: i32, 
    y: i32, 
    width: i32, 
    height: i32,
    parent: Option<HWND>,
    menu: Option<HMENU>,
    instance: Option<HINSTANCE>
) -> Result<HWND, ()> {
    let mut class_name_w = class_name.encode_utf16().collect::<Vec<_>>();
    class_name_w.push(0x0000);
    let mut window_name_w = window_name.encode_utf16().collect::<Vec<_>>();
    window_name_w.push(0x0000);

    let res = unsafe {
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
    if res.0 == 0 {
        Err(())
    } else {
        Ok(res)
    }
}

#[test]
fn cw_test() {
    assert!(
        create_window(
            "test_class",
            "test_window",
            None,
            None,
            0,
            0,
            600,
            400,
            None,
            None,
            get_instance()
        ).is_ok()
    );
}

impl Window {
    pub fn new() -> Self {
        todo!()
    }
}