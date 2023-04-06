#![allow(dead_code)]

use core::slice;
use std::{
    collections::HashMap,
    mem::{size_of, transmute},
    ptr::{addr_of, addr_of_mut},
    sync::{atomic::AtomicU16, Arc, RwLock},
    thread,
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32WindowHandle};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
        Graphics::Gdi::{RedrawWindow, UpdateWindow, COLOR_WINDOW, HBRUSH, RDW_NOINTERNALPAINT},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{
                GetActiveWindow, MapVirtualKeyW, SetFocus, ToUnicode, MAPVK_VK_TO_CHAR,
                MAPVK_VSC_TO_VK_EX, VIRTUAL_KEY, VK_ADD, VK_BACK, VK_CAPITAL,
                VK_CONTROL, VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1,
                VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9,
                VK_HOME, VK_INSERT, VK_LBUTTON, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN,
                VK_MBUTTON, VK_MENU, VK_MULTIPLY, VK_NEXT, VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1,
                VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8,
                VK_NUMPAD9, VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7,
                VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_PAUSE, VK_PRIOR,
                VK_RBUTTON, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN,
                VK_SEPARATOR, VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_TAB, VK_UP,
                VK_XBUTTON1, VK_XBUTTON2,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FlashWindowEx,
                GetSystemMetrics, GetWindowLongPtrW, LoadCursorW, LoadIconW, PeekMessageW,
                PostMessageW, RegisterClassExW, SendMessageW, SetWindowLongPtrW, SetWindowPos,
                SetWindowTextW, ShowWindow, CS_DBLCLKS, CS_NOCLOSE, CW_USEDEFAULT, FLASHWINFO,
                FLASHW_ALL, FLASHW_TIMERNOFG, FLASHW_TRAY, GWL_EXSTYLE, GWL_STYLE, HCURSOR, HICON,
                HMENU, HWND_TOP, IDC_ARROW, IDI_APPLICATION, MINMAXINFO, MSG, PM_REMOVE,
                SC_MAXIMIZE, SC_NEXTWINDOW, SC_RESTORE, SIZE_MAXHIDE, SIZE_MAXIMIZED, SIZE_MAXSHOW,
                SIZE_MINIMIZED, SIZE_RESTORED, SM_CXSCREEN, SM_CYSCREEN, SWP_ASYNCWINDOWPOS,
                SWP_DRAWFRAME, SWP_FRAMECHANGED, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOCOPYBITS,
                SWP_SHOWWINDOW, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_NORMAL, WA_ACTIVE,
                WA_CLICKACTIVE, WA_INACTIVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATE, WM_CLOSE,
                WM_DESTROY, WM_DISPLAYCHANGE, WM_GETMINMAXINFO, WM_KEYDOWN, WM_KEYUP, WM_MOVE,
                WM_SETTEXT, WM_SIZE, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW,
                WNDCLASS_STYLES, WS_CLIPSIBLINGS, WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
                WS_OVERLAPPEDWINDOW, WS_POPUP, WS_SIZEBOX, WS_VISIBLE, WM_CREATE, WM_MOUSEWHEEL,
            },
        },
    },
};

use crate::{
    EventSender, FullscreenType, KeyboardScancode, Modifiers, MouseScancode, Theme,
    UserAttentionType, WindowButtons, WindowEvent, WindowId, WindowIdExt, WindowSizeState,
    WindowTExt,
};

#[derive(Clone, Debug, Default)]
pub struct Window {
    hwnd: Arc<HWND>,
}

#[derive(Clone, Debug)]
pub(crate) struct WindowInfo {
    hinstance: HINSTANCE,
    visible: bool,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    min_width: i32,
    min_height: i32,
    max_width: i32,
    max_height: i32,
    parent: Option<HWND>,
    icon: HICON,
    icon_small: HICON,
    menu: Option<HMENU>,
    menu_name: String,
    style: WINDOW_STYLE,
    style_ex: WINDOW_EX_STYLE,
    class_name: String,
    class_id: WndClassId,
    title: String,
    cursor: HCURSOR,
    background: HBRUSH,
    no_close: bool,
    focused: bool,
    resizeable: bool,
    theme: Theme,
    has_frame: bool,
    fullscreen: FullscreenType,
    non_fullscreen_style: WINDOW_STYLE,
    size_state: WindowSizeState,
    enabled_buttons: WindowButtons,
    modifiers: Modifiers,
    sender: Arc<RwLock<EventSender>>,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            width: CW_USEDEFAULT,
            height: CW_USEDEFAULT,
            style: WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS,
            style_ex: WS_EX_APPWINDOW,
            class_name: "nwin default".to_owned(),
            hinstance: get_instance().unwrap(),
            title: "nwin window".to_owned(),
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            visible: false,
            min_width: 20,
            max_width: unsafe { GetSystemMetrics(SM_CXSCREEN) } as _,
            min_height: 20,
            max_height: unsafe { GetSystemMetrics(SM_CYSCREEN) } as _,
            parent: None,
            icon: unsafe { LoadIconW(None, IDI_APPLICATION).unwrap() },
            icon_small: unsafe { LoadIconW(None, IDI_APPLICATION).unwrap() },
            menu: None,
            menu_name: "nwin menu".to_owned(),
            class_id: WndClassId(0),
            cursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
            background: HBRUSH(COLOR_WINDOW.0 as isize + 1),
            no_close: false,
            focused: false,
            resizeable: true,
            theme: Theme::Light,
            has_frame: false,
            fullscreen: FullscreenType::NotFullscreen,
            non_fullscreen_style: WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS,
            size_state: WindowSizeState::Other,
            enabled_buttons: WindowButtons::all(),
            modifiers: Modifiers::empty(),
            sender: Arc::new(RwLock::new(EventSender::new())),
        }
    }
}

static CLASS_ID: AtomicU16 = AtomicU16::new(0);

impl WindowInfo {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register(&mut self) -> Result<WndClassId, WIN32_ERROR> {
        let res = register_class(
            &self.menu_name,
            &self.class_name,
            Some(self.icon),
            Some(self.icon_small),
            Some(self.cursor),
            Some(self.background),
            self.no_close,
        );

        if let Ok(id) = res {
            CLASS_ID.store(id.0, std::sync::atomic::Ordering::Relaxed);
        }
        

        res
    }

    pub(crate) fn create(&mut self) -> Result<HWND, WIN32_ERROR> {
        create_window(
            &self.class_name,
            &self.title,
            self.visible,
            Some(self.style_ex),
            Some(self.style),
            self.x,
            self.y,
            self.width,
            self.height,
            self.parent,
            self.menu,
            self.hinstance,
        )
    }
}

lazy_static::lazy_static! {
    static ref WINDOW_INFO: Arc<RwLock<HashMap<isize, WindowInfo>>> = Arc::new(RwLock::new(HashMap::new()));
}

impl Window {
    pub fn try_new() -> Result<Self, WIN32_ERROR> {
        let mut info = WindowInfo::new();
        assert_eq!(info.style, WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS);
        let class_id = if CLASS_ID.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            info.register()?
        } else {
            WndClassId(CLASS_ID.load(std::sync::atomic::Ordering::Relaxed))
        };
        info.class_id = class_id;
        let hwnd = info.create()?;
        assert_eq!(
            info.style,
            WINDOW_STYLE(unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as _)
        );

        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(hwnd.0)
            .and_modify(|v| *v = info.clone())
            .or_insert(info);
        assert_eq!(
            WINDOW_INFO
                .clone()
                .read()
                .unwrap()
                .get(&hwnd.0)
                .unwrap()
                .style,
            WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS
        );
        Ok(Self {
            hwnd: Arc::new(hwnd),
        })
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if Arc::strong_count(&self.hwnd) <= 1 {
            WINDOW_INFO.clone().write().unwrap().remove(&self.hwnd.0);
        }
    }
}

impl WindowIdExt for WindowId {
    fn next_event(&self) {
        let mut msg = MSG::default();
        if unsafe { PeekMessageW(addr_of_mut!(msg), HWND(self.0 as _), 0, 0, PM_REMOVE) }.as_bool()
        {
            unsafe { DispatchMessageW(addr_of_mut!(msg)) };
        }
    }
}

fn get_instance() -> Option<HINSTANCE> {
    unsafe { GetModuleHandleW(None).ok() }
}

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub(crate) struct WndClassId(u16);

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

fn register_class(
    menu_name: &str,
    class_name: &str,
    icon: Option<HICON>,
    icon_small: Option<HICON>,
    cursor: Option<HCURSOR>,
    background: Option<HBRUSH>,
    no_close: bool,
) -> Result<WndClassId, WIN32_ERROR> {
    let close = if no_close {
        CS_NOCLOSE
    } else {
        WNDCLASS_STYLES(0)
    };
    let mut menu_name_w = menu_name.encode_utf16().collect::<Vec<_>>();
    menu_name_w.push(0x0000);
    let mut class_name_w = class_name.encode_utf16().collect::<Vec<_>>();
    class_name_w.push(0x0000);

    let wndclass = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_DBLCLKS | close,
        lpfnWndProc: Some(main_wnd_proc),
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

#[allow(clippy::too_many_arguments)]
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
    hinstance: HINSTANCE,
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
            style.unwrap_or(WINDOW_STYLE(0)) | WS_CLIPSIBLINGS,
            x,
            y,
            width,
            height,
            parent.unwrap_or(HWND(0)),
            menu.unwrap_or(HMENU(0)),
            hinstance,
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

impl TryFrom<VIRTUAL_KEY> for KeyboardScancode {
    type Error = ();
    fn try_from(value: VIRTUAL_KEY) -> Result<Self, Self::Error> {
        match value {
            VK_BACK => Ok(Self::Backspace),
            VK_TAB => Ok(Self::Tab),
            VK_RETURN => Ok(Self::Enter),
            VK_PAUSE => Ok(Self::PauseBreak),
            VK_ESCAPE => Ok(Self::Esc),
            VK_SPACE => Ok(Self::Space),
            VK_PRIOR => Ok(Self::PgUp),
            VK_NEXT => Ok(Self::PgDn),
            VK_END => Ok(Self::End),
            VK_HOME => Ok(Self::Home),
            VK_LEFT => Ok(Self::ArrowLeft),
            VK_UP => Ok(Self::ArrowUp),
            VK_DOWN => Ok(Self::ArrowDown),
            VK_RIGHT => Ok(Self::ArrowRight),
            VK_SNAPSHOT => Ok(Self::PrtScSysRq),
            VK_INSERT => Ok(Self::Insert),
            VK_DELETE => Ok(Self::Del),
            VIRTUAL_KEY(0x30) => Ok(Self::Key0),
            VIRTUAL_KEY(0x31) => Ok(Self::Key1),
            VIRTUAL_KEY(0x32) => Ok(Self::Key2),
            VIRTUAL_KEY(0x33) => Ok(Self::Key3),
            VIRTUAL_KEY(0x34) => Ok(Self::Key4),
            VIRTUAL_KEY(0x35) => Ok(Self::Key5),
            VIRTUAL_KEY(0x36) => Ok(Self::Key6),
            VIRTUAL_KEY(0x37) => Ok(Self::Key7),
            VIRTUAL_KEY(0x38) => Ok(Self::Key8),
            VIRTUAL_KEY(0x39) => Ok(Self::Key9),
            VIRTUAL_KEY(0x41) => Ok(Self::A),
            VIRTUAL_KEY(0x42) => Ok(Self::B),
            VIRTUAL_KEY(0x43) => Ok(Self::C),
            VIRTUAL_KEY(0x44) => Ok(Self::D),
            VIRTUAL_KEY(0x45) => Ok(Self::E),
            VIRTUAL_KEY(0x46) => Ok(Self::F),
            VIRTUAL_KEY(0x47) => Ok(Self::G),
            VIRTUAL_KEY(0x48) => Ok(Self::H),
            VIRTUAL_KEY(0x49) => Ok(Self::I),
            VIRTUAL_KEY(0x4A) => Ok(Self::J),
            VIRTUAL_KEY(0x4B) => Ok(Self::K),
            VIRTUAL_KEY(0x4C) => Ok(Self::L),
            VIRTUAL_KEY(0x4D) => Ok(Self::M),
            VIRTUAL_KEY(0x4E) => Ok(Self::N),
            VIRTUAL_KEY(0x4F) => Ok(Self::O),
            VIRTUAL_KEY(0x50) => Ok(Self::P),
            VIRTUAL_KEY(0x51) => Ok(Self::Q),
            VIRTUAL_KEY(0x52) => Ok(Self::R),
            VIRTUAL_KEY(0x53) => Ok(Self::S),
            VIRTUAL_KEY(0x54) => Ok(Self::T),
            VIRTUAL_KEY(0x55) => Ok(Self::U),
            VIRTUAL_KEY(0x56) => Ok(Self::V),
            VIRTUAL_KEY(0x57) => Ok(Self::W),
            VIRTUAL_KEY(0x58) => Ok(Self::X),
            VIRTUAL_KEY(0x59) => Ok(Self::Y),
            VIRTUAL_KEY(0x5A) => Ok(Self::Z),
            VK_NUMPAD0 => Ok(Self::Num0),
            VK_NUMPAD1 => Ok(Self::Num1),
            VK_NUMPAD2 => Ok(Self::Num2),
            VK_NUMPAD3 => Ok(Self::Num3),
            VK_NUMPAD4 => Ok(Self::Num4),
            VK_NUMPAD5 => Ok(Self::Num5),
            VK_NUMPAD6 => Ok(Self::Num6),
            VK_NUMPAD7 => Ok(Self::Num7),
            VK_NUMPAD8 => Ok(Self::Num8),
            VK_NUMPAD9 => Ok(Self::Num9),
            VK_MULTIPLY => Ok(Self::NumAsterisk),
            VK_ADD => Ok(Self::NumPlus),
            VK_SEPARATOR => Ok(Self::NumPeriod),
            VK_SUBTRACT => Ok(Self::NumHyphen),
            VK_DECIMAL => Ok(Self::NumPeriod),
            VK_DIVIDE => Ok(Self::NumSlash),
            VK_F1 => Ok(Self::F1),
            VK_F2 => Ok(Self::F2),
            VK_F3 => Ok(Self::F3),
            VK_F4 => Ok(Self::F4),
            VK_F5 => Ok(Self::F5),
            VK_F6 => Ok(Self::F6),
            VK_F7 => Ok(Self::F7),
            VK_F8 => Ok(Self::F8),
            VK_F9 => Ok(Self::F9),
            VK_F10 => Ok(Self::F10),
            VK_F11 => Ok(Self::F11),
            VK_F12 => Ok(Self::F12),

            VK_OEM_1 => Ok(Self::Semicolon),
            VK_OEM_PLUS => Ok(Self::Equals),
            VK_OEM_COMMA => Ok(Self::Comma),
            VK_OEM_MINUS => Ok(Self::Hyphen),
            VK_OEM_PERIOD => Ok(Self::Period),
            VK_OEM_2 => Ok(Self::ForwardSlash),
            VK_OEM_3 => Ok(Self::Tilde),
            VK_OEM_4 => Ok(Self::OpenBracket),
            VK_OEM_5 => Ok(Self::BackSlash),
            VK_OEM_6 => Ok(Self::CloseBracket),
            VK_OEM_7 => Ok(Self::Apostrophe),

            _ => Err(()),
        }
    }
}

impl TryFrom<VIRTUAL_KEY> for MouseScancode {
    type Error = ();
    fn try_from(value: VIRTUAL_KEY) -> Result<Self, Self::Error> {
        match value {
            VK_LBUTTON => Ok(Self::LClick),
            VK_RBUTTON => Ok(Self::RClick),
            VK_MBUTTON => Ok(Self::MClick),
            VK_XBUTTON1 => Ok(Self::Button4),
            VK_XBUTTON2 => Ok(Self::Button5),
            _ => Err(()),
        }
    }
}

trait ModifiersExt {
    fn try_from_vk(vk: VIRTUAL_KEY, scancode: u16) -> Option<Modifiers>;
}

impl ModifiersExt for Modifiers {
    fn try_from_vk(vk: VIRTUAL_KEY, scancode: u16) -> Option<Self> {
        let vk = if vk == VK_SHIFT || vk == VK_MENU || vk == VK_CONTROL {
            VIRTUAL_KEY(unsafe { MapVirtualKeyW(scancode as _, MAPVK_VSC_TO_VK_EX) } as _)
        } else {
            vk
        };

        match vk {
            VK_LSHIFT => Some(Modifiers::LSHIFT),
            VK_RSHIFT => Some(Modifiers::RSHIFT),
            VK_LMENU => Some(Modifiers::LALT),
            VK_RMENU => Some(Modifiers::RALT),
            VK_LCONTROL => Some(Modifiers::LCTRL),
            VK_RCONTROL => Some(Modifiers::RCTRL),
            VK_LWIN => Some(Modifiers::LSYS),
            VK_RWIN => Some(Modifiers::RSYS),
            VK_CAPITAL => Some(Modifiers::CAPSLOCK),
            VK_NUMLOCK => Some(Modifiers::NUMLOCK),
            _ => None,
        }
    }
}
enum KeyState {
    Up,
    Down,
}

impl KeyState {
    fn from_bool(b: bool) -> Self {
        if b {
            Self::Down
        } else {
            Self::Up
        }
    }
}

struct KeyPressInfo {
    repeat_count: u16,
    scancode: u16,
    context_code: bool,
    previous_state: KeyState,
}

impl KeyPressInfo {
    fn from_lparam(lparam: LPARAM) -> Self {
        Self::from_isize(lparam.0)
    }

    fn from_isize(i: isize) -> Self {
        let repeat_count = (i & 0x0000FFFF) as u16;
        let scancode = ((i & 0x00FF0000) >> 16) as u16;
        let extended = i & 0x01000000 != 0;
        let scancode = if extended {
            scancode | 0xE000
        } else { 
            scancode
        };
        let context_code = i & 0x10000000 != 0;
        let previous_state = KeyState::from_bool(i & 0x40000000 != 0);

        Self {
            repeat_count,
            scancode,
            context_code,
            previous_state,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct OemScancode(u16);

impl TryFrom<OemScancode> for KeyboardScancode {
    type Error = ();
    fn try_from(value: OemScancode) -> Result<Self, Self::Error> {
        match value.0 {
            0x001E => Ok(Self::A),
            0x0030 => Ok(Self::B),
            0x002E => Ok(Self::C),
            0x0020 => Ok(Self::D),
            0x0012 => Ok(Self::E),
            0x0021 => Ok(Self::F),
            0x0022 => Ok(Self::G),
            0x0023 => Ok(Self::H),
            0x0017 => Ok(Self::I),
            0x0024 => Ok(Self::J),
            0x0025 => Ok(Self::K),
            0x0026 => Ok(Self::L),
            0x0032 => Ok(Self::M),
            0x0031 => Ok(Self::N),
            0x0018 => Ok(Self::O),
            0x0019 => Ok(Self::P),
            0x0010 => Ok(Self::Q),
            0x0013 => Ok(Self::R),
            0x001F => Ok(Self::S),
            0x0014 => Ok(Self::T),
            0x0016 => Ok(Self::U),
            0x002F => Ok(Self::V),
            0x0011 => Ok(Self::W),
            0x002D => Ok(Self::X),
            0x0015 => Ok(Self::Y),
            0x002C => Ok(Self::Z),
            
            0x0002 => Ok(Self::Key1),
            0x0003 => Ok(Self::Key2),
            0x0004 => Ok(Self::Key3),
            0x0005 => Ok(Self::Key4),
            0x0006 => Ok(Self::Key5),
            0x0007 => Ok(Self::Key6),
            0x0008 => Ok(Self::Key7),
            0x0009 => Ok(Self::Key8),
            0x000A => Ok(Self::Key9),
            0x000B => Ok(Self::Key0),
            
            0x001C => Ok(Self::Enter),
            0x0001 => Ok(Self::Esc),
            0x000E => Ok(Self::Backspace),
            0x000F => Ok(Self::Tab),

            0x0039 => Ok(Self::Space),
            0x000C => Ok(Self::Hyphen),
            0x000D => Ok(Self::Equals),
            0x001A => Ok(Self::OpenBracket),
            0x001B => Ok(Self::CloseBracket),
            0x002B => Ok(Self::BackSlash),
            0x0027 => Ok(Self::Semicolon),
            0x0028 => Ok(Self::Apostrophe),
            0x0029 => Ok(Self::Tilde),
            0x0033 => Ok(Self::Comma),
            0x0034 => Ok(Self::Period),
            0x0035 => Ok(Self::ForwardSlash),
            0x003A => Ok(Self::CapsLk),

            0x003B => Ok(Self::F1),
            0x003C => Ok(Self::F2),
            0x003D => Ok(Self::F3),
            0x003E => Ok(Self::F4),
            0x003F => Ok(Self::F5),
            0x0040 => Ok(Self::F6),
            0x0041 => Ok(Self::F7),
            0x0042 => Ok(Self::F8),
            0x0043 => Ok(Self::F9),
            0x0044 => Ok(Self::F10),
            0x0057 => Ok(Self::F11),
            0x0058 => Ok(Self::F12),

            0x0046 => Ok(Self::ScrLk),
            0xE052 => Ok(Self::Insert),
            0xE047 => Ok(Self::Home),
            0xE049 => Ok(Self::PgUp),
            0xE053 => Ok(Self::Del),
            0xE04F => Ok(Self::End),
            0xE051 => Ok(Self::PgDn),
            0xE04D => Ok(Self::ArrowRight),
            0xE04B => Ok(Self::ArrowLeft),
            0xE050 => Ok(Self::ArrowDown),
            0xE048 => Ok(Self::ArrowUp),
            
            0xE035 => Ok(Self::NumSlash),
            0x0037 => Ok(Self::NumAsterisk),
            0x004A => Ok(Self::NumHyphen),
            0x004E => Ok(Self::NumPlus),
            0xE01C => Ok(Self::NumEnter),
            0x0053 => Ok(Self::NumPeriod),

            0x004F => Ok(Self::Num1),
            0x0050 => Ok(Self::Num2),
            0x0051 => Ok(Self::Num3),
            0x004B => Ok(Self::Num4),
            0x004C => Ok(Self::Num5),
            0x004D => Ok(Self::Num6),
            0x0047 => Ok(Self::Num7),
            0x0048 => Ok(Self::Num8),
            0x0049 => Ok(Self::Num9),
            0x0052 => Ok(Self::Num0),

            0x001D => Ok(Self::LCtrl),
            0x002A => Ok(Self::LShift),
            0x0038 => Ok(Self::LAlt),
            0xE05B => Ok(Self::LSys),
            0xE01D => Ok(Self::RCtrl),
            0x0036 => Ok(Self::RShift),
            0xE038 => Ok(Self::RAlt),
            0xE05C => Ok(Self::RSys),

            _ => Err(())
        }
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(hwnd.0)
            .or_insert(WindowInfo::default())
            .sender
                    .read()
                    .unwrap()
                    .send(WindowId(hwnd.0 as _), WindowEvent::Created); 
        }
        WM_CLOSE => {
            WINDOW_INFO
                .clone()
                .write()
                .unwrap()
                .entry(hwnd.0)
                .and_modify(|info| {
                    info.sender
                        .read()
                        .unwrap()
                        .send(WindowId(hwnd.0 as _), WindowEvent::CloseRequested);
                })
                .or_insert(WindowInfo::default());
            DestroyWindow(hwnd);
        }
        WM_DESTROY => {
            PostMessageW(hwnd, msg, wparam, lparam);
            WINDOW_INFO
                .clone()
                .write()
                .unwrap()
                .entry(hwnd.0)
                .and_modify(|info| {
                    info.sender
                        .read()
                        .unwrap()
                        .send(WindowId(hwnd.0 as _), WindowEvent::Destroyed);
                })
                .or_insert(WindowInfo::default());
            WINDOW_INFO.clone().write().unwrap().remove(&hwnd.0);
            return LRESULT(0);
        }
        WM_GETMINMAXINFO => {
            let mmi = lparam.0 as *mut MINMAXINFO;
            let lock = WINDOW_INFO.clone();
            let mut info = lock.write().unwrap();
            let entry = info.entry(hwnd.0).or_default();
            (*mmi).ptMinTrackSize.x = entry.min_height;
            (*mmi).ptMinTrackSize.y = entry.min_height;
            (*mmi).ptMaxTrackSize.x = entry.max_width;
            (*mmi).ptMaxTrackSize.y = entry.max_height;
            return LRESULT(0);
        }
        WM_MOVE => {
            let x = lparam.0 & 0xFFFF;
            let y = (lparam.0 >> 16) & 0xFFFF;
            WINDOW_INFO
                .clone()
                .write()
                .unwrap()
                .entry(hwnd.0)
                .and_modify(|info| {
                    info.x = x as _;
                    info.y = y as _;
                    info.sender.read().unwrap().send(
                        WindowId(hwnd.0 as _),
                        WindowEvent::Moved {
                            x: x as _,
                            y: y as _,
                        },
                    );
                })
                .or_insert(WindowInfo::default());
            return LRESULT(0);
        }
        WM_SIZE => {
            let width = lparam.0 & 0xFFFF;
            let height = (lparam.0 >> 16) & 0xFFFF;
            match wparam.0 as u32 {
                SIZE_RESTORED => {
                    WINDOW_INFO
                        .clone()
                        .write()
                        .unwrap()
                        .entry(hwnd.0)
                        .and_modify(|info| {
                            info.width = width as _;
                            info.height = height as _;
                            info.size_state = WindowSizeState::Other;
                            info.sender.read().unwrap().send(
                                WindowId(hwnd.0 as _),
                                WindowEvent::Resized {
                                    width: width as _,
                                    height: height as _,
                                },
                            );
                        });

                    return LRESULT(0);
                }
                SIZE_MINIMIZED => {
                    WINDOW_INFO
                        .clone()
                        .write()
                        .unwrap()
                        .entry(hwnd.0)
                        .and_modify(|v| {
                            v.size_state = WindowSizeState::Minimized;
                        });
                    return LRESULT(0);
                }
                SIZE_MAXIMIZED => {
                    WINDOW_INFO
                        .clone()
                        .write()
                        .unwrap()
                        .entry(hwnd.0)
                        .and_modify(|v| {
                            v.size_state = WindowSizeState::Maximized;
                        });

                    return LRESULT(0);
                }
                SIZE_MAXSHOW | SIZE_MAXHIDE => todo!(),
                _ => return LRESULT(0),
            }
        }
        WM_ACTIVATE => {
            let focused = match wparam.0 as u32 {
                WA_ACTIVE | WA_CLICKACTIVE => true,
                WA_INACTIVE => false,
                _ => return LRESULT(0),
            };

            WINDOW_INFO
                .clone()
                .write()
                .unwrap()
                .entry(hwnd.0)
                .and_modify(|info| {
                    info.focused = focused;
                    info.sender
                        .read()
                        .unwrap()
                        .send(WindowId(hwnd.0 as _), WindowEvent::Focused(focused));
                });
            return LRESULT(0);
        },
        WM_SETTEXT => {
            let text = lparam.0 as *mut u16;
            let mut len = 1;
            while unsafe { *text.add(len) } != 0x0000 {
                len += 1;
            }
            let v = slice::from_raw_parts(text, len);
            if let Ok(s) = String::from_utf16(v) {
                WINDOW_INFO
                    .clone()
                    .write()
                    .unwrap()
                    .entry(hwnd.0)
                    .and_modify(|v| v.title = s);
            };
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
        WM_DISPLAYCHANGE => todo!(),
        WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            let sys = msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP;
            let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let kpi = KeyPressInfo::from_lparam(lparam);
            let vk = VIRTUAL_KEY(wparam.0 as _);
            let physical_scancode: Option<KeyboardScancode> = OemScancode(kpi.scancode).try_into().ok();

            if sys && (vk == VK_TAB || vk == VK_RETURN) {
                let info = WINDOW_INFO
                    .clone()
                    .write()
                    .unwrap()
                    .get(&hwnd.0)
                    .unwrap()
                    .clone();
                let wparam = if vk == VK_RETURN {
                    if info.size_state == WindowSizeState::Maximized {
                        WPARAM(SC_RESTORE as _)
                    } else {
                        WPARAM(SC_MAXIMIZE as _)
                    }
                } else {
                    WPARAM(SC_NEXTWINDOW as _)
                };
                unsafe { SendMessageW(hwnd, WM_SYSCOMMAND, wparam, LPARAM(0)) };
                return LRESULT(0);
            }

            if let Ok(k) = TryInto::<KeyboardScancode>::try_into(vk) {
                WINDOW_INFO
                    .clone()
                    .write()
                    .unwrap()
                    .entry(hwnd.0)
                    .and_modify(|v| {
                        if !down {
                            v.sender.clone().write().unwrap().send(
                                WindowId(hwnd.0 as _),
                                WindowEvent::KeyUp {
                                    logical_scancode: k,
                                    physical_scancode,
                                },
                            );
                            return;
                        }

                        let c = unsafe { MapVirtualKeyW(vk.0 as _, MAPVK_VK_TO_CHAR) };
                        let unshifted_char = std::char::decode_utf16([c as u16])
                            .flatten()
                            .collect::<Vec<_>>()
                            .iter()
                            .copied()
                            .nth(0);

                        let mut keystate = [0u8; 256];
                        let b = v.modifiers.contains(Modifiers::LSHIFT)
                            || v.modifiers.contains(Modifiers::RSHIFT);
                        let b = if v.modifiers.contains(Modifiers::CAPSLOCK) {
                            !b
                        } else {
                            b
                        };
                        if b {
                            keystate[0x10] = 0x80;
                        }
                        let mut buf = [0u16; 1];
                        let res = unsafe {
                            ToUnicode(
                                (vk.0 & 0xFF) as _,
                                (vk.0 & 0xFF) as _,
                                Some(&keystate),
                                &mut buf,
                                0,
                            )
                        };
                        let character = if res != 1 {
                            None
                        } else {
                            std::char::decode_utf16(buf)
                                .flatten()
                                .collect::<Vec<_>>()
                                .iter()
                                .copied()
                                .nth(0)
                        };

                        v.sender.clone().write().unwrap().send(
                            WindowId(hwnd.0 as _),
                            WindowEvent::KeyDown {
                                logical_scancode: k,
                                character,
                                unshifted_char,
                                physical_scancode,
                            },
                        )
                    });
            }

            if let Ok(k) = TryInto::<MouseScancode>::try_into(vk) {
                WINDOW_INFO
                    .clone()
                    .write()
                    .unwrap()
                    .entry(hwnd.0)
                    .and_modify(|v| {
                        v.sender
                            .clone()
                            .write()
                            .unwrap()
                            .send(WindowId(hwnd.0 as _), if down { WindowEvent::MouseButtonDown(k) } else { WindowEvent::MouseButtonUp(k) })
                    });
            }

            if let Some(k) = Modifiers::try_from_vk(vk, kpi.scancode) {
                WINDOW_INFO
                    .clone()
                    .write()
                    .unwrap()
                    .entry(hwnd.0)
                    .and_modify(|v| {
                        if k == Modifiers::CAPSLOCK || k == Modifiers::NUMLOCK {
                            if down {
                                v.modifiers ^= k;
                            } else{

                            }
                        } else if down {
                            v.modifiers |= k;
                        } else if !down {
                            v.modifiers &= !k;
                        }

                        v.sender.clone().write().unwrap().send(
                            WindowId(hwnd.0 as _),
                            WindowEvent::ModifiersChanged(v.modifiers),
                        )
                    })
                    .or_insert(WindowInfo::default());
            }

            return LRESULT(0);
        },
        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 & 0xFFFF0000) >> 16) as i16;
            WINDOW_INFO
                .clone()
                .write()
                .unwrap()
                .entry(hwnd.0)
                .and_modify(|v| {
                    v.sender
                        .clone()
                        .write()
                        .unwrap()
                        .send(WindowId(hwnd.0 as _), WindowEvent::MouseWheelScroll(delta as _));
                });
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    };
    LRESULT(0)
}

fn minimize_window(hwnd: HWND) {
    if WINDOW_INFO
        .clone()
        .read()
        .unwrap()
        .get(&hwnd.0)
        .unwrap()
        .size_state
        != WindowSizeState::Minimized
    {
        unsafe {
            ShowWindow(hwnd, SW_MINIMIZE);
        }
    }
}

fn maximize_window(hwnd: HWND) {
    if WINDOW_INFO
        .clone()
        .read()
        .unwrap()
        .get(&hwnd.0)
        .unwrap()
        .size_state
        != WindowSizeState::Maximized
    {
        unsafe {
            ShowWindow(hwnd, SW_MAXIMIZE);
        }
    }
}

impl super::super::WindowT for Window {
    fn id(&self) -> WindowId {
        WindowId(unsafe { transmute(self.hwnd.0 as i64) })
    }

    fn focus(&mut self) {
        if unsafe { GetActiveWindow() } == HWND(self.hwnd.0) {
            return;
        }

        unsafe {
            SetFocus(HWND(self.hwnd.0));
        }
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.focused = true);
    }

    fn focused(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .focused
    }

    fn width(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .width as _
    }

    fn min_width(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .min_width as _
    }

    fn max_width(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .max_width as _
    }

    fn set_width(&mut self, width: u32) {
        let lock = WINDOW_INFO.clone();
        lock.write().unwrap().entry(self.hwnd.0).and_modify(|v| {
            v.width = width as _;
            let mut flags = SWP_NOACTIVATE;
            if v.has_frame {
                flags |= SWP_DRAWFRAME;
            }
            flags |= if v.visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };
            unsafe {
                SetWindowPos(*self.hwnd, HWND_TOP, v.x, v.y, v.width, v.height, flags);
            }
        });
    }

    fn set_min_width(&mut self, width: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.min_width = width as _);
    }

    fn set_max_width(&mut self, width: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.max_width = width as _);
    }

    fn height(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .height as _
    }

    fn min_height(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .min_height as _
    }

    fn max_height(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .max_height as _
    }

    fn set_height(&mut self, height: u32) {
        let lock = WINDOW_INFO.clone();
        lock.write().unwrap().entry(self.hwnd.0).and_modify(|v| {
            v.height = height as _;
            let mut flags = SWP_NOACTIVATE;
            if v.has_frame {
                flags |= SWP_DRAWFRAME;
            }
            flags |= if v.visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };
            unsafe {
                SetWindowPos(*self.hwnd, HWND_TOP, v.x, v.y, v.width, v.height, flags);
            }
        });
    }

    fn set_min_height(&mut self, height: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.min_height = height as _);
    }

    fn set_max_height(&mut self, height: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.max_height = height as _);
    }

    fn visible(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .visible
    }

    fn show(&mut self) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| {
                v.visible = true;
                v.style |= WS_VISIBLE;
            });
        unsafe {
            ShowWindow(*self.hwnd, SW_NORMAL);
        }
    }

    fn hide(&mut self) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.visible = false);
        unsafe {
            ShowWindow(*self.hwnd, SW_HIDE);
        }
    }

    fn resizeable(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .resizeable
    }

    fn set_resizeable(&mut self, resizeable: bool) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| v.resizeable = resizeable);
        unsafe {
            SetWindowLongPtrW(
                *self.hwnd,
                GWL_STYLE,
                GetWindowLongPtrW(*self.hwnd, GWL_STYLE) & !WS_SIZEBOX.0 as isize,
            )
        };
    }

    fn theme(&self) -> Theme {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .theme
    }

    fn set_theme(&mut self, _theme: Theme) {
        todo!()
    }

    fn title(&self) -> String {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .title
            .clone()
    }

    fn fullscreen(&self) -> bool {
        let fullscreen = WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .fullscreen;
        fullscreen == FullscreenType::Exclusive || fullscreen == FullscreenType::Borderless
    }

    fn fullscreen_type(&self) -> FullscreenType {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: FullscreenType) {
        if WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .fullscreen
            == fullscreen
        {
            return;
        }

        let lock = WINDOW_INFO.clone();
        let mut info = lock.write().unwrap();

        info.entry(self.hwnd.0).and_modify(|v| {
            let mut flags = SWP_NOACTIVATE | SWP_FRAMECHANGED;
            if v.has_frame {
                flags |= SWP_DRAWFRAME;
            }
            flags |= if v.visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };

            if fullscreen == FullscreenType::Borderless {
                v.non_fullscreen_style =
                    WINDOW_STYLE(unsafe { GetWindowLongPtrW(*self.hwnd, GWL_STYLE) } as _);
                if v.non_fullscreen_style.contains(WS_POPUP) {
                    let style = WS_VISIBLE | WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS;
                    unsafe {
                        SetWindowLongPtrW(*self.hwnd, GWL_STYLE, style.0 as _);
                    }
                    v.style = style;
                    unsafe {
                        SetWindowPos(*self.hwnd, None, 0, 0, 600, 400, flags);
                    }
                } else {
                    let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                    let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
                    let style = WS_VISIBLE | WS_POPUP;
                    unsafe {
                        SetWindowLongPtrW(*self.hwnd, GWL_STYLE, style.0 as isize);
                    }
                    v.style = style;
                    unsafe {
                        SetWindowPos(*self.hwnd, HWND_TOP, 0, 0, w, h, flags);
                    }
                }
            } else if fullscreen == FullscreenType::Exclusive {
                todo!()
            } else {
                unsafe {
                    SetWindowLongPtrW(*self.hwnd, GWL_STYLE, v.non_fullscreen_style.0 as _);
                }
                unsafe {
                    SetWindowPos(*self.hwnd, HWND_TOP, v.x, v.y, v.width, v.height, flags);
                }
            }
        });
    }

    fn maximized(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .size_state
            == WindowSizeState::Maximized
    }

    fn minimized(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .size_state
            == WindowSizeState::Minimized
    }

    fn normalized(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .size_state
            == WindowSizeState::Other
    }

    fn maximize(&mut self) {
        maximize_window(*self.hwnd)
    }

    fn minimize(&mut self) {
        minimize_window(*self.hwnd);
    }

    fn normalize(&mut self) {
        let info = WINDOW_INFO
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .clone();
        if info.size_state != WindowSizeState::Minimized {
            let mut flags = SWP_FRAMECHANGED | SWP_ASYNCWINDOWPOS | SWP_NOCOPYBITS;
            if info.has_frame {
                flags |= SWP_DRAWFRAME;
            }
            flags |= if info.visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };
            unsafe {
                SetWindowPos(
                    *self.hwnd,
                    HWND_TOP,
                    info.x,
                    info.y,
                    info.width,
                    info.height,
                    flags,
                );
            }
        }
    }

    fn request_user_attention(&mut self, attention: UserAttentionType) {
        let hwnd = *self.hwnd;
        if unsafe { GetActiveWindow() } == hwnd {
            return;
        }

        thread::spawn(move || {
            let flags = if attention == UserAttentionType::Critical {
                FLASHW_ALL | FLASHW_TIMERNOFG
            } else {
                FLASHW_TRAY | FLASHW_TIMERNOFG
            };

            let count = if attention == UserAttentionType::Critical {
                u32::MAX
            } else {
                0
            };

            let wi = FLASHWINFO {
                cbSize: size_of::<FLASHWINFO>() as _,
                hwnd,
                dwFlags: flags,
                uCount: count,
                dwTimeout: 0,
            };

            unsafe {
                FlashWindowEx(addr_of!(wi));
            }
        });
    }

    fn request_redraw(&mut self) {
        unsafe {
            RedrawWindow(*self.hwnd, None, None, RDW_NOINTERNALPAINT);
        }
    }

    fn enabled_buttons(&self) -> WindowButtons {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .enabled_buttons
    }

    fn set_enabled_buttons(&mut self, buttons: WindowButtons) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.hwnd.0)
            .and_modify(|v| {
                v.enabled_buttons = buttons;
                let mut style = WINDOW_STYLE(0);
                if buttons.contains(WindowButtons::MAXIMIZE) {
                    style |= WS_MAXIMIZEBOX
                };
                if buttons.contains(WindowButtons::MINIMIZE) {
                    style |= WS_MINIMIZEBOX
                };
                v.style &= !style;

                unsafe {
                    SetWindowLongPtrW(*self.hwnd, GWL_STYLE, v.style.0 as _);
                }

                if v.no_close == false && buttons.contains(WindowButtons::CLOSE) {
                    return;
                }

                todo!()
            });
    }
}

impl WindowTExt for Window {
    fn sender(&self) -> Arc<RwLock<EventSender>> {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .sender
            .clone()
    }
}

pub trait WindowExtWindows {
    fn style(&self) -> WINDOW_STYLE;
    fn set_style(&mut self, style: WINDOW_STYLE);
    fn set_style_ex(&mut self, style_ex: WINDOW_EX_STYLE);
    fn set_title(&mut self, title: &str);
}

impl WindowExtWindows for Window {
    fn style(&self) -> WINDOW_STYLE {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .style
    }

    fn set_style(&mut self, style: WINDOW_STYLE) {
        let mut info = WINDOW_INFO.write().unwrap();
        let info = info.get_mut(&self.hwnd.0).unwrap();
        info.style = style | WS_CLIPSIBLINGS;
        info.non_fullscreen_style = style | WS_CLIPSIBLINGS;
        unsafe { SetWindowLongPtrW(*self.hwnd, GWL_STYLE, style.0 as _) };
        unsafe { UpdateWindow(*self.hwnd) };
    }

    fn set_style_ex(&mut self, style_ex: WINDOW_EX_STYLE) {
        let mut info = WINDOW_INFO.write().unwrap();
        let info = info.get_mut(&self.hwnd.0).unwrap();
        info.style_ex = style_ex;
        unsafe { SetWindowLongPtrW(*self.hwnd, GWL_EXSTYLE, style_ex.0 as _) };
        unsafe { UpdateWindow(*self.hwnd) };
    }

    fn set_title(&mut self, title: &str) {
        let mut title_w = title.encode_utf16().collect::<Vec<_>>();
        title_w.push(0x0000);

        unsafe {
            SetWindowTextW(*self.hwnd, PCWSTR(title_w.as_ptr())).unwrap();
        }
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = Win32WindowHandle::empty();
        let hinstance = WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&self.hwnd.0)
            .unwrap()
            .hinstance;
        handle.hinstance = hinstance.0 as _;
        handle.hwnd = self.hwnd.0 as _;
        RawWindowHandle::Win32(handle)
    }
}

mod tests {
    //#[test]
    fn cw_test() {
        use crate::platform::win32::{create_window, get_instance, register_class};
        use std::ptr::{addr_of, addr_of_mut};
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, MSG,
        };
        use windows::Win32::UI::WindowsAndMessaging::{CW_USEDEFAULT, WS_OVERLAPPEDWINDOW};

        let class_name = "test_class";

        let _class_id =
            register_class("test_menu", class_name, None, None, None, None, false).unwrap();

        let hwnd = create_window(
            class_name,
            "test_window",
            true,
            None,
            Some(WS_OVERLAPPEDWINDOW),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            get_instance().unwrap(),
        )
        .unwrap();

        let mut msg = MSG::default();
        println!("running message loop!");
        loop {
            if unsafe { GetMessageW(addr_of_mut!(msg), hwnd, 0, 0).0 <= 0 } {
                break;
            }

            unsafe { TranslateMessage(addr_of!(msg)) };
            unsafe { DispatchMessageW(addr_of!(msg)) };
        }
    }

    // #[test]
    fn w_test() {
        use crate::platform::*;
        use std::ptr::{addr_of, addr_of_mut};

        use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWL_STYLE, WINDOW_STYLE};
        use windows::Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
        };

        use crate::platform::win32::WindowExtWindows;

        use crate::WindowT;

        let mut window = win32::Window::try_new().unwrap();
        window.show();

        let hwnd = HWND(window.id().0 as _);
        let style = WINDOW_STYLE(unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32);
        assert_eq!(style, window.style());
        let mut msg = MSG::default();
        println!("running message loop!");
        loop {
            if unsafe { GetMessageW(addr_of_mut!(msg), hwnd, 0, 0).0 <= 0 } {
                break;
            }

            unsafe { TranslateMessage(addr_of!(msg)) };
            unsafe { DispatchMessageW(addr_of!(msg)) };
        }
    }

    //#[test]
    fn w_test_no_decorations() {
        use crate::platform::*;
        use std::ptr::{addr_of, addr_of_mut};

        use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWL_STYLE, WINDOW_STYLE};
        use windows::Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, TranslateMessage, MSG, WS_POPUP,
            },
        };

        use crate::platform::win32::WindowExtWindows;

        use crate::WindowT;

        let mut window = win32::Window::try_new().unwrap();
        window.set_style(WS_POPUP);
        window.show();

        let hwnd = HWND(window.id().0 as _);
        let style = WINDOW_STYLE(unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32);
        assert_eq!(style, window.style());
        let mut msg = MSG::default();
        loop {
            if unsafe { GetMessageW(addr_of_mut!(msg), hwnd, 0, 0).0 <= 0 } {
                break;
            }

            unsafe { TranslateMessage(addr_of!(msg)) };
            unsafe { DispatchMessageW(addr_of!(msg)) };
        }
    }
}
