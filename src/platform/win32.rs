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
            Input::KeyboardAndMouse::{GetActiveWindow, SetFocus},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FlashWindowEx,
                GetSystemMetrics, GetWindowLongPtrW, LoadCursorW, LoadIconW, PeekMessageW,
                PostMessageW, RegisterClassExW, SetWindowLongPtrW, SetWindowPos, SetWindowTextW,
                ShowWindow, CS_DBLCLKS, CS_NOCLOSE, CW_USEDEFAULT, FLASHWINFO, FLASHW_ALL,
                FLASHW_TIMERNOFG, FLASHW_TRAY, GWL_EXSTYLE, GWL_STYLE, HCURSOR, HICON, HMENU,
                HWND_TOP, IDC_ARROW, IDI_APPLICATION, MINMAXINFO, MSG, PM_REMOVE, SIZE_MAXHIDE,
                SIZE_MAXIMIZED, SIZE_MAXSHOW, SIZE_MINIMIZED, SIZE_RESTORED, SM_CXSCREEN,
                SM_CYSCREEN, SWP_ASYNCWINDOWPOS, SWP_DRAWFRAME, SWP_FRAMECHANGED, SWP_HIDEWINDOW,
                SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_SHOWWINDOW, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE,
                SW_NORMAL, WA_ACTIVE, WA_CLICKACTIVE, WA_INACTIVE, WINDOW_EX_STYLE, WINDOW_STYLE,
                WM_ACTIVATE, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE, WM_GETMINMAXINFO, WM_MOVE,
                WM_SETTEXT, WM_SIZE, WNDCLASSEXW, WNDCLASS_STYLES, WS_CLIPSIBLINGS,
                WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPEDWINDOW, WS_POPUP,
                WS_SIZEBOX, WS_VISIBLE,
            },
        },
    },
};

use crate::{
    EventSender, FullscreenType, Theme, UserAttentionType, WindowButtons, WindowEvent, WindowId,
    WindowIdExt, WindowSizeState, WindowTExt,
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

        CLASS_ID.store(self.class_id.0, std::sync::atomic::Ordering::Relaxed);

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

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
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
                    info.sender
                        .read()
                        .unwrap()
                        .send(WindowId(hwnd.0 as _), WindowEvent::Moved(x as _, y as _));
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
                                WindowEvent::Resized(width as _, height as _),
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
        }
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
            return LRESULT(0);
        }
        WM_DISPLAYCHANGE => todo!(),
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
            SetWindowTextW(*self.hwnd, PCWSTR(title_w.as_ptr()));
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
