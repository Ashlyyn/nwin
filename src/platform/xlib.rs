#![allow(dead_code, non_upper_case_globals)]

use core::slice;
use std::{
    collections::HashMap,
    ffi::CString,
    mem::MaybeUninit,
    ptr::addr_of_mut,
    sync::{
        atomic::{AtomicU32, AtomicU64},
        Arc, RwLock,
    },
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, XlibWindowHandle};
use x11::xlib::{
    Always, Button1, Button1MotionMask, Button2, Button2MotionMask, Button3, Button3MotionMask,
    Button4, Button4MotionMask, Button5, Button5MotionMask, ButtonMotionMask, ButtonPress,
    ButtonPressMask, ButtonRelease, ButtonReleaseMask, CWBackPixel, CWBackPixmap, CWBackingPixel,
    CWBackingPlanes, CWBackingStore, CWBitGravity, CWBorderPixel, CWBorderPixmap, CWColormap,
    CWCursor, CWDontPropagate, CWEventMask, CWOverrideRedirect, CWSaveUnder, CWWinGravity,
    CenterGravity, ClientMessage, ClientMessageData, Colormap, ColormapChangeMask, ConfigureNotify,
    ControlMask, CopyFromParent, CurrentTime, Cursor, DestroyNotify, EastGravity, EnterWindowMask,
    ExposureMask, FocusChangeMask, FocusIn, FocusOut, ForgetGravity, InputOnly, InputOutput,
    KeyPress, KeyPressMask, KeyRelease, KeyReleaseMask, KeymapStateMask, LeaveWindowMask, LockMask,
    Mod1Mask, Mod4Mask, NorthEastGravity, NorthGravity, NorthWestGravity, NotUseful,
    OwnerGrabButtonMask, PMaxSize, PMinSize, Pixmap, PointerMotionHintMask, PointerMotionMask,
    PropertyChangeMask, ResizeRedirectMask, RevertToParent, ShiftMask, SouthEastGravity,
    SouthGravity, SouthWestGravity, StaticGravity, StructureNotifyMask, SubstructureNotifyMask,
    SubstructureRedirectMask, VisibilityChangeMask, Visual, VisualAllMask, WestGravity, WhenMapped,
    XAllocSizeHints, XCheckWindowEvent, XClientMessageEvent, XCloseDisplay, XCreateWindow,
    XDefaultRootWindow, XDefaultScreen, XDestroyWindow, XEvent, XFree, XGetVisualInfo,
    XIconifyWindow, XInternAtom, XMapWindow, XMatchVisualInfo, XOpenDisplay, XRaiseWindow,
    XResizeWindow, XRootWindow, XSelectInput, XSendEvent, XSetInputFocus, XSetWMNormalHints,
    XSetWindowAttributes, XStoreName, XUnmapWindow, XVisualInfo,
};

use crate::{
    EventSender, FullscreenType, Modifiers, MouseButtons, Theme, WindowButtons, WindowId,
    WindowIdExt, WindowSizeState, WindowTExt,
};

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
#[repr(u32)]
enum WindowClass {
    InputOnly = InputOnly as _,
    InputOutput = InputOutput as _,
    #[default]
    CopyFromParent = CopyFromParent as _,
}

impl WindowClass {
    pub fn as_u32(&self) -> u32 {
        *self as _
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(i32)]
pub enum Gravity {
    Forget = ForgetGravity,
    Static = StaticGravity,
    NorthWest = NorthWestGravity,
    North = NorthGravity,
    NorthEast = NorthEastGravity,
    West = WestGravity,
    Center = CenterGravity,
    East = EastGravity,
    SouthWest = SouthWestGravity,
    South = SouthGravity,
    SouthEast = SouthEastGravity,
}

impl Gravity {
    pub fn as_i32(&self) -> i32 {
        *self as _
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(i32)]
pub enum BackingStore {
    NotUseful = NotUseful,
    WhenMapped = WhenMapped,
    Always = Always,
}

impl BackingStore {
    pub fn as_i32(&self) -> i32 {
        *self as _
    }
}

pub struct BackingPlanes(u64);

bitflags::bitflags! {
    #[derive(Copy, Clone, Default, Debug)]
    pub struct EventMask: i64 {
        const KEY_PRESS = KeyPressMask as _;
        const KEY_RELEASE = KeyReleaseMask as _;
        const BUTTON_PRESS = ButtonPressMask as _;
        const BUTTON_RELEASE = ButtonReleaseMask as _;
        const ENTER_WINDOW = EnterWindowMask as _;
        const LEAVE_WINDOW = LeaveWindowMask as _;
        const POINTER_MOTION = PointerMotionMask as _;
        const POINTER_MOTION_HINT = PointerMotionHintMask as _;
        const BUTTON_1_MOTION = Button1MotionMask as _;
        const BUTTON_2_MOTION = Button2MotionMask as _;
        const BUTTON_3_MOTION = Button3MotionMask as _;
        const BUTTON_4_MOTION = Button4MotionMask as _;
        const BUTTON_5_MOTION = Button5MotionMask as _;
        const BUTTON_MOTION = ButtonMotionMask as _;
        const KEYMAP_STATE = KeymapStateMask as _;
        const EXPOSURE = ExposureMask as _;
        const VISIBILITY_CHANGE = VisibilityChangeMask as _;
        const STRUCTURE_NOTIFY = StructureNotifyMask as _;
        const RESIZE_REDIRECT = ResizeRedirectMask as _;
        const SUBSTRUCTURE_NOTIFY = SubstructureNotifyMask as _;
        const SUBSTRUCTURE_REDIRECT = SubstructureRedirectMask as _;
        const FOCUS_CHANGE = FocusChangeMask as _;
        const PROPERTY_CHANGE = PropertyChangeMask as _;
        const COLORMAP_CHANGE = ColormapChangeMask as _;
        const OWNER_GRAB_BUTTON_MASK = OwnerGrabButtonMask as _;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WindowAttributes {
    inner: XSetWindowAttributes,
    mask: u64,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            inner: XSetWindowAttributes {
                background_pixmap: 0,
                background_pixel: 0,
                border_pixmap: CopyFromParent as _,
                border_pixel: 0,
                bit_gravity: ForgetGravity,
                win_gravity: NorthWestGravity,
                backing_store: NotUseful,
                backing_planes: !0,
                backing_pixel: 0,
                save_under: x11::xlib::False,
                event_mask: 0,
                do_not_propagate_mask: 0,
                override_redirect: x11::xlib::False,
                colormap: CopyFromParent as _,
                cursor: 0,
            },
            mask: 0,
        }
    }
}

pub struct WindowAttributesBuilder {
    inner: WindowAttributes,
}

impl WindowAttributesBuilder {
    pub fn new() -> Self {
        Self {
            inner: WindowAttributes {
                inner: unsafe { MaybeUninit::zeroed().assume_init() },
                mask: 0,
            },
        }
    }

    pub fn with_background_pixmap(mut self, pixmap: Pixmap) -> Self {
        self.inner.inner.background_pixmap = pixmap;
        self.inner.mask |= CWBackPixmap;
        self
    }

    pub fn with_background_pixel(mut self, pixel: u64) -> Self {
        self.inner.inner.background_pixel = pixel;
        self.inner.mask |= CWBackPixel;
        self
    }

    pub fn with_border_pixmap(mut self, pixmap: Pixmap) -> Self {
        self.inner.inner.border_pixmap = pixmap;
        self.inner.mask |= CWBorderPixmap;
        self
    }

    pub fn with_border_pixel(mut self, pixel: u64) -> Self {
        self.inner.inner.border_pixel = pixel;
        self.inner.mask |= CWBorderPixel;
        self
    }

    pub fn with_bit_gravity(mut self, gravity: Gravity) -> Self {
        self.inner.inner.bit_gravity = gravity.as_i32();
        self.inner.mask |= CWBitGravity;
        self
    }

    pub fn with_win_gravity(mut self, gravity: Gravity) -> Self {
        self.inner.inner.win_gravity = gravity.as_i32();
        self.inner.mask |= CWWinGravity;
        self
    }

    pub fn with_backing_store(mut self, backing_store: BackingStore) -> Self {
        self.inner.inner.backing_store = backing_store.as_i32();
        self.inner.mask |= CWBackingStore;
        self
    }

    pub fn with_backing_planes(mut self, planes: BackingPlanes) -> Self {
        self.inner.inner.backing_planes = planes.0;
        self.inner.mask |= CWBackingPlanes;
        self
    }

    pub fn with_backing_pixel(mut self, pixel: u64) -> Self {
        self.inner.inner.backing_pixel = pixel;
        self.inner.mask |= CWBackingPixel;
        self
    }

    pub fn with_save_under(mut self, save_under: bool) -> Self {
        self.inner.inner.save_under = save_under as _;
        self.inner.mask |= CWSaveUnder;
        self
    }

    pub fn with_event_mask(mut self, mask: EventMask) -> Self {
        self.inner.inner.event_mask = mask.bits();
        self.inner.mask |= CWEventMask;
        self
    }

    pub fn with_do_not_propagate_mask(mut self, mask: EventMask) -> Self {
        self.inner.inner.do_not_propagate_mask = mask.bits();
        self.inner.mask |= CWDontPropagate;
        self
    }

    pub fn with_override_redirect(mut self, redirect: bool) -> Self {
        self.inner.inner.override_redirect = redirect as _;
        self.inner.mask |= CWOverrideRedirect;
        self
    }

    pub fn with_colormap(mut self, colormap: Colormap) -> Self {
        self.inner.inner.colormap = colormap;
        self.inner.mask |= CWColormap;
        self
    }

    pub fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.inner.inner.cursor = cursor;
        self.inner.mask |= CWCursor;
        self
    }

    pub fn build(self) -> WindowAttributes {
        self.inner
    }
}

#[allow(clippy::too_many_arguments)]
fn create_window(
    window_name: &str,
    parent: Option<x11::xlib::Window>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    visible: bool,
    border_width: u32,
    depth: Option<i32>,
    class: WindowClass,
    attributes: Option<WindowAttributes>,
    event_mask: EventMask,
) -> Result<
    (
        x11::xlib::Window,
        *mut x11::xlib::Display,
        i32,
        x11::xlib::VisualID,
    ),
    (),
> {
    let display = unsafe { XOpenDisplay(core::ptr::null()) };
    if display.is_null() {
        return Err(());
    }

    let screen = unsafe { XDefaultScreen(display) };

    let mut vinfo: XVisualInfo = unsafe { MaybeUninit::zeroed().assume_init() };
    vinfo.class = class.as_u32() as _;
    vinfo.screen = screen;
    vinfo.depth = depth.unwrap_or(0);
    let (visual, visual_id) = if unsafe {
        XMatchVisualInfo(
            display,
            screen,
            depth.unwrap_or(0),
            class.as_u32() as _,
            addr_of_mut!(vinfo),
        )
    } == 0
    {
        let mut nitems = 0i32;
        let p = unsafe {
            XGetVisualInfo(
                display,
                VisualAllMask,
                addr_of_mut!(vinfo),
                addr_of_mut!(nitems),
            )
        };
        let ret = if nitems == 0 {
            (core::ptr::null_mut(), 0)
        } else {
            let vi = unsafe { slice::from_raw_parts(p, nitems as _) };
            (vi[0].visual, vi[0].visualid)
        };
        unsafe { XFree(p.cast()) };
        ret
    } else {
        (vinfo.visual, vinfo.visualid)
    };

    let mask = if let Some(ref a) = attributes {
        a.mask
    } else {
        0
    };
    let attributes = if let Some(mut a) = attributes {
        addr_of_mut!(a.inner)
    } else {
        core::ptr::null_mut()
    };

    let window = unsafe {
        XCreateWindow(
            display,
            parent.unwrap_or_else(|| XRootWindow(display, XDefaultScreen(display))),
            x,
            y,
            width,
            height,
            border_width,
            depth.unwrap_or(CopyFromParent as _),
            class.as_u32(),
            visual,
            mask,
            attributes,
        )
    };
    assert_ne!(window, 0);

    if window < 16 {
        return Err(());
    }

    unsafe { XSelectInput(display, window, event_mask.bits()) };
    if visible {
        unsafe {
            XMapWindow(display, window);
        }
    };
    let window_name_c = CString::new(window_name).unwrap();
    unsafe { XStoreName(display, window, window_name_c.as_ptr()) };
    Ok((window, display, screen, visual_id))
}

mod tests {
    /*
    use crate::WindowT;

    //#[test]
    fn cw_test() {
        use std::{mem::MaybeUninit, ptr::addr_of_mut};
        use x11::xlib::{XEvent, XNextEvent, KeyPress};
        use super::{create_window, WindowClass, EventMask};
        use x11::xlib::{XDestroyWindow};

        let (id, display, _screen, _visual_id) = create_window(
            "test window", None, 0, 0, 600, 400, true, 10,
            None, WindowClass::InputOutput,
            None, EventMask::all()
        ).unwrap();

        let mut event: XEvent = unsafe { MaybeUninit::zeroed().assume_init() };
        loop {
            unsafe { XNextEvent(display, addr_of_mut!(event)) };
            match event.get_type() {
                KeyPress => break,
                _ => { },
           }
        }
        unsafe { XDestroyWindow(display, id) };
    }

    //#[test]
    fn cw_test_2() {
        use std::{mem::MaybeUninit, ptr::addr_of_mut};
        use x11::xlib::{XEvent, XNextEvent, XDestroyWindow};
        use super::create_window;
        use x11::xlib::KeyPress;

        let (id, display, _screen, _visual_id) = create_window(
            "nwin window",
            None,
            0,
            0,
            640,
            480,
            true,
            10,
            None,
            super::WindowClass::InputOutput,
            None,
            super::EventMask::all()
        ).unwrap();

        let mut event: XEvent = unsafe { MaybeUninit::zeroed().assume_init() };
        loop {
            unsafe { XNextEvent(display, addr_of_mut!(event)) };
            match event.get_type() {
                KeyPress => break,
                _ => { },
           }
        }
        unsafe { XDestroyWindow(display, id) };
    }

    #[test]
    fn w_test() {
        use std::{mem::MaybeUninit, ptr::addr_of_mut};
        use x11::xlib::{KeyPress, XEvent, XNextEvent};
        use x11::xlib::XClearWindow;
        use crate::platform::xlib::{WindowExtXlib, EventMask};
        use x11::xlib::{FocusIn, FocusOut, MapNotify, UnmapNotify, ReparentNotify, ConfigureNotify, ResizeRequest};

        let mut window = super::Window::try_new(None, None).unwrap();
        assert_ne!(window.id().0, 0);
        window.set_resizeable(false);
        window.show();
        window.set_event_mask(EventMask::KEY_PRESS | EventMask::FOCUS_CHANGE | EventMask::VISIBILITY_CHANGE | EventMask::STRUCTURE_NOTIFY);
        let mut event: XEvent = unsafe { MaybeUninit::zeroed().assume_init() };
        loop {
            unsafe { XClearWindow(window.display, *window.id) };
            unsafe { XNextEvent(window.display, addr_of_mut!(event)) };
            if unsafe { event.any.window } == *window.id {
                match event.get_type() {
                    FocusIn => {
                        window.focused = true;
                    },
                    FocusOut => {
                        window.focused = false;
                    },
                    MapNotify => {
                        window.visible = true;
                    },
                    UnmapNotify => {
                        window.visible = false;
                    },
                    ReparentNotify => {
                        window.parent = unsafe { event.reparent.parent };
                    },
                    ConfigureNotify => {
                        let cfg = unsafe { event.configure };
                        window.x = cfg.x;
                        window.y = cfg.y;
                        window.width = cfg.width as _;
                        window.height = cfg.height as _;
                        window.border_width = cfg.border_width as _;
                    },
                    ResizeRequest => {
                        let cfg = unsafe { event.resize_request };
                        window.height = cfg.width as _;
                        window.width = cfg.height as _;
                    },
                    KeyPress => break,
                    _ => { }
               }
            }
        }
    }
    */
}

#[derive(Clone, Debug, Default)]
pub struct Window {
    id: Arc<x11::xlib::Window>,
}

#[derive(Clone, Debug)]
pub(crate) struct WindowInfo {
    display: *mut x11::xlib::Display,
    visual_id: x11::xlib::VisualID,
    name: String,
    screen: i32,
    parent: x11::xlib::Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
    visible: bool,
    border_width: u32,
    depth: i32,
    class: WindowClass,
    visual: Option<Visual>,
    event_mask: EventMask,
    enabled_buttons: WindowButtons,
    focused: bool,
    fullscreen: FullscreenType,
    size_state: WindowSizeState,
    resizeable: bool,
    theme: Theme,
    modifiers: Modifiers,
    sender: Arc<RwLock<EventSender>>,
}

unsafe impl Send for WindowInfo {}
unsafe impl Sync for WindowInfo {}

lazy_static::lazy_static! {
    static ref WINDOW_INFO: Arc<RwLock<HashMap<x11::xlib::XID, WindowInfo>>> = Arc::new(RwLock::new(HashMap::new()));
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            display: core::ptr::null_mut(),
            visual_id: 0,
            name: "nwin window".to_owned(),
            parent: 0,
            screen: 0,
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            min_width: 20,
            min_height: 20,
            max_width: u32::MAX,
            max_height: u32::MAX,
            visible: false,
            border_width: 10,
            depth: CopyFromParent as _,
            class: WindowClass::InputOutput,
            visual: None,
            event_mask: EventMask::all(),
            enabled_buttons: WindowButtons::all(),
            focused: false,
            fullscreen: FullscreenType::NotFullscreen,
            size_state: WindowSizeState::Other,
            resizeable: false,
            theme: Theme::Light,
            modifiers: Modifiers::empty(),
            sender: Arc::new(RwLock::new(EventSender::new())),
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if Arc::strong_count(&self.id) <= 1 {
            WINDOW_INFO.clone().write().unwrap().remove(&*self.id);
            //unsafe { XDestroyWindow(w.display, *self.id) };
        }
    }
}

impl Window {
    pub fn try_new(
        parent: Option<x11::xlib::Window>,
        attributes: Option<WindowAttributes>,
    ) -> Result<Self, ()> {
        let mut w = Self::default();
        let mut info = WindowInfo::default();
        let (id, display, screen, visual_id) = w.create(parent, attributes, &info)?;
        w.id = Arc::new(id);
        info.display = display;
        info.screen = screen;
        info.visual_id = visual_id;
        info.parent = parent.unwrap_or(unsafe { XRootWindow(display, info.screen) });
        WINDOW_INFO.clone().write().unwrap().insert(id, info);
        let wm_delete_window_s = CString::new("WM_DELETE_WINDOW").unwrap();
        let wm_delete_window =
            unsafe { XInternAtom(display, wm_delete_window_s.as_ptr(), x11::xlib::True) };
        WM_DELETE_WINDOW.store(wm_delete_window, std::sync::atomic::Ordering::Relaxed);
        Ok(w)
    }

    fn create(
        &self,
        parent: Option<x11::xlib::Window>,
        attributes: Option<WindowAttributes>,
        w: &WindowInfo,
    ) -> Result<
        (
            x11::xlib::Window,
            *mut x11::xlib::Display,
            i32,
            x11::xlib::VisualID,
        ),
        (),
    > {
        create_window(
            &w.name,
            parent,
            w.x,
            w.y,
            w.width,
            w.height,
            w.visible,
            w.border_width,
            Some(w.depth),
            w.class,
            attributes,
            w.event_mask,
        )
    }
}

impl crate::WindowT for Window {
    fn enabled_buttons(&self) -> crate::WindowButtons {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .enabled_buttons
    }

    fn set_enabled_buttons(&mut self, buttons: WindowButtons) {
        /*
        let allowed_actions_s = CString::new("_NET_WM_ALLOWED_ACTIONS").unwrap();
        let maximize_horz_s = CString::new("_NET_WM_ACTION_MAXIMIZE_HORZ").unwrap();
        let maximize_vert_s = CString::new("_NET_WM_ACTION_MAXIMIZE_VERT").unwrap();

        let allowed_actions = unsafe { XInternAtom(w.display, allowed_actions_s.as_ptr(), x11::xlib::False) };
        let maximize_horz = unsafe { XInternAtom(w.display, maximize_horz_s.as_ptr(), x11::xlib::False) };
        let maximize_vert = unsafe { XInternAtom(w.display, maximize_vert_s.as_ptr(), x11::xlib::False) };

        unsafe { XChangeProperty(w.display, *self.id, allowed_actions, XA_ATOM, 32, PropModeAppend, addr_of_mut!(maximize_horz) as _, 1) }
        */
        if buttons != WindowButtons::all() {
            todo!()
        }
    }

    fn focus(&mut self) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.focused = true;
                unsafe { XSetInputFocus(w.display, *self.id, RevertToParent, CurrentTime) };
                unsafe { XRaiseWindow(w.display, *self.id) };
            })
            .or_insert(WindowInfo::default());
    }

    fn focused(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .focused
    }

    fn fullscreen_type(&self) -> FullscreenType {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .fullscreen
    }

    fn width(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .width
    }

    fn set_width(&mut self, width: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.width = width;
                unsafe { XResizeWindow(w.display, *self.id, w.width, w.height) };
            })
            .or_insert(WindowInfo::default());
    }

    fn height(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .height
    }

    fn set_height(&mut self, height: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.height = height;
                unsafe { XResizeWindow(w.display, *self.id, w.width, w.height) };
            })
            .or_insert(WindowInfo::default());
    }

    fn id(&self) -> WindowId {
        WindowId(*self.id as _)
    }

    fn min_width(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .min_width
    }

    fn set_min_width(&mut self, width: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.min_width = width;
                let size_hints = &mut unsafe { *XAllocSizeHints() };
                size_hints.min_width = w.min_width as _;
                size_hints.min_height = w.min_height as _;
                size_hints.flags = PMinSize;
                unsafe { XSetWMNormalHints(w.display, *self.id, addr_of_mut!(*size_hints)) };
                unsafe { XFree(addr_of_mut!(*size_hints) as _) };
            })
            .or_insert(WindowInfo::default());
    }

    fn min_height(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .min_height
    }

    fn set_min_height(&mut self, height: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.min_height = height;
                let size_hints = &mut unsafe { *XAllocSizeHints() };
                size_hints.min_width = w.min_width as _;
                size_hints.min_height = w.min_height as _;
                size_hints.flags = PMinSize;
                unsafe { XSetWMNormalHints(w.display, *self.id, addr_of_mut!(*size_hints)) };
                unsafe { XFree(addr_of_mut!(*size_hints) as _) };
            })
            .or_insert(WindowInfo::default());
    }

    fn max_width(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .max_width
    }

    fn set_max_width(&mut self, width: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.max_width = width;
                let size_hints = &mut unsafe { *XAllocSizeHints() };
                size_hints.min_width = w.min_width as _;
                size_hints.min_height = w.min_height as _;
                size_hints.flags = PMinSize;
                unsafe { XSetWMNormalHints(w.display, *self.id, addr_of_mut!(*size_hints)) };
                unsafe { XFree(addr_of_mut!(*size_hints) as _) };
            })
            .or_insert(WindowInfo::default());
    }

    fn max_height(&self) -> u32 {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .max_height
    }

    fn set_max_height(&mut self, height: u32) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.max_height = height;
                let size_hints = &mut unsafe { *XAllocSizeHints() };
                size_hints.min_width = w.min_width as _;
                size_hints.min_height = w.min_height as _;
                size_hints.flags = PMinSize;
                unsafe { XSetWMNormalHints(w.display, *self.id, addr_of_mut!(*size_hints)) };
                unsafe { XFree(addr_of_mut!(*size_hints) as _) };
            })
            .or_insert(WindowInfo::default());
    }

    fn maximized(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .size_state
            == WindowSizeState::Maximized
    }

    fn maximize(&mut self) {
        const NET_WM_TOGGLE_STATE: i64 = 2;

        let wm_state_s = CString::new("_NET_WM_STATE").unwrap();
        let max_width_s = CString::new("_NET_WM_STATE_MAXIMIZED_HORZ").unwrap();
        let max_height_s = CString::new("_NET_WM_STATE_MAXIMIZED_VERT").unwrap();

        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                let wm_state =
                    unsafe { XInternAtom(w.display, wm_state_s.as_ptr(), x11::xlib::False) };
                let max_width =
                    unsafe { XInternAtom(w.display, max_width_s.as_ptr(), x11::xlib::False) };
                let max_height =
                    unsafe { XInternAtom(w.display, max_height_s.as_ptr(), x11::xlib::False) };

                let mut ev = XClientMessageEvent {
                    type_: ClientMessage,
                    format: 32,
                    window: *self.id,
                    message_type: wm_state,
                    data: ClientMessageData::from([
                        NET_WM_TOGGLE_STATE,
                        max_width as _,
                        max_height as _,
                        1,
                        0,
                    ]),
                    serial: 0,
                    send_event: 0,
                    display: w.display,
                };

                unsafe {
                    XSendEvent(
                        w.display,
                        XDefaultRootWindow(w.display),
                        x11::xlib::False,
                        SubstructureNotifyMask,
                        addr_of_mut!(ev) as _,
                    )
                };
                w.size_state = WindowSizeState::Maximized;
            })
            .or_insert(WindowInfo::default());
    }

    fn minimized(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .size_state
            == WindowSizeState::Minimized
    }

    fn minimize(&mut self) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                unsafe { XIconifyWindow(w.display, *self.id, w.screen) };
                w.size_state = WindowSizeState::Minimized;
            })
            .or_insert(WindowInfo::default());
    }

    fn normalized(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .size_state
            == WindowSizeState::Other
    }

    // TODO - implement better
    fn normalize(&mut self) {
        if self.maximized() {
            self.maximize();
        } else {
            self.maximize();
            self.maximize();
        }

        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.size_state = WindowSizeState::Other;
            })
            .or_insert(WindowInfo::default());
    }

    fn resizeable(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .resizeable
    }

    fn set_resizeable(&mut self, resizeable: bool) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.resizeable = resizeable;
                let size_hints = &mut unsafe { *XAllocSizeHints() };
                if resizeable == false {
                    size_hints.min_width = w.width as _;
                    size_hints.max_width = w.width as _;
                    size_hints.min_height = w.height as _;
                    size_hints.max_height = w.height as _;
                } else {
                    size_hints.min_width = w.min_width as _;
                    size_hints.max_width = w.max_width as _;
                    size_hints.min_height = w.min_height as _;
                    size_hints.max_height = w.min_height as _;
                }
                size_hints.flags = PMinSize | PMaxSize;
                unsafe { XSetWMNormalHints(w.display, *self.id, addr_of_mut!(*size_hints)) };
            })
            .or_insert(WindowInfo::default());
    }

    fn theme(&self) -> Theme {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .theme
    }

    fn set_theme(&mut self, theme: Theme) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .get_mut(&*self.id)
            .unwrap()
            .theme = theme;
        todo!()
    }

    fn title(&self) -> String {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .name
            .clone()
    }

    fn visible(&self) -> bool {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .visible
    }

    fn hide(&mut self) {
        unsafe {
            XUnmapWindow(
                WINDOW_INFO
                    .clone()
                    .read()
                    .unwrap()
                    .get(&*self.id)
                    .unwrap()
                    .display,
                *self.id,
            )
        };
    }

    fn show(&mut self) {
        unsafe {
            XMapWindow(
                WINDOW_INFO
                    .clone()
                    .read()
                    .unwrap()
                    .get(&*self.id)
                    .unwrap()
                    .display,
                *self.id,
            )
        };
    }

    fn request_redraw(&mut self) {
        todo!()
    }

    fn request_user_attention(&mut self, _attention: crate::UserAttentionType) {
        todo!()
    }

    fn set_fullscreen(&mut self, _fullscreen: FullscreenType) {
        todo!()
    }
}

trait WindowExtXlib {
    fn event_mask(&self) -> EventMask;
    fn set_event_mask(&mut self, event_mask: EventMask);
    fn set_title(&mut self, title: &str);
}

impl WindowExtXlib for Window {
    fn event_mask(&self) -> EventMask {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .event_mask
    }

    fn set_event_mask(&mut self, event_mask: EventMask) {
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(*self.id)
            .and_modify(|w| {
                w.event_mask = event_mask;
                unsafe { XSelectInput(w.display, *self.id, event_mask.bits()) };
            })
            .or_insert(WindowInfo::default());
    }

    fn set_title(&mut self, title: &str) {
        let title_c = CString::new(title).unwrap();
        unsafe {
            XStoreName(
                WINDOW_INFO
                    .clone()
                    .read()
                    .unwrap()
                    .get(&*self.id)
                    .unwrap()
                    .display,
                *self.id,
                title_c.as_ptr(),
            )
        };
    }
}

impl WindowTExt for Window {
    fn sender(&self) -> Arc<RwLock<EventSender>> {
        WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .sender
            .clone()
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = XlibWindowHandle::empty();
        handle.window = *self.id;
        handle.visual_id = WINDOW_INFO
            .clone()
            .read()
            .unwrap()
            .get(&*self.id)
            .unwrap()
            .visual_id;
        RawWindowHandle::Xlib(handle)
    }
}

static WM_DELETE_WINDOW: AtomicU64 = AtomicU64::new(0);

impl WindowIdExt for WindowId {
    fn next_event(&self) {
        let mut ev: XEvent = unsafe { MaybeUninit::zeroed().assume_init() };
        WINDOW_INFO
            .clone()
            .write()
            .unwrap()
            .entry(self.0)
            .and_modify(|w| {
                if unsafe {
                    XCheckWindowEvent(
                        w.display,
                        self.0 as _,
                        w.event_mask.bits(),
                        addr_of_mut!(ev),
                    )
                } == x11::xlib::False
                {
                    return;
                }

                match unsafe { ev.type_ } {
                    DestroyNotify => {
                        w.sender
                            .write()
                            .unwrap()
                            .send(WindowId(self.0), crate::WindowEvent::CloseRequested);
                        w.sender
                            .write()
                            .unwrap()
                            .send(WindowId(self.0), crate::WindowEvent::Destroyed);
                    }
                    ConfigureNotify => {
                        let cfg = unsafe { ev.configure };
                        if cfg.x != w.x || cfg.y != w.y {
                            w.x = cfg.x;
                            w.y = cfg.y;
                            w.sender.write().unwrap().send(
                                WindowId(self.0),
                                crate::WindowEvent::Moved(w.x as _, w.y as _),
                            );
                        } else if cfg.width != w.width as _ || cfg.height != w.height as _ {
                            w.width = cfg.width as _;
                            w.height = cfg.height as _;
                            w.sender.write().unwrap().send(
                                WindowId(self.0),
                                crate::WindowEvent::Resized(w.width, w.height),
                            );
                        }
                    }
                    KeyPress => {
                        let kp = unsafe { ev.key };
                        w.sender.write().unwrap().send(
                            WindowId(self.0),
                            crate::WindowEvent::KeyDown(crate::KeyboardInput {
                                key_code: kp.keycode as _,
                            }),
                        );

                        let modifiers =
                            kp.state & (ShiftMask | ControlMask | Mod1Mask | Mod4Mask | LockMask);
                        let mut m = Modifiers::empty();
                        if modifiers & ShiftMask != 0 {
                            m |= Modifiers::LSHIFT;
                        }
                        if modifiers & ControlMask != 0 {
                            m |= Modifiers::LCTRL;
                        }
                        if modifiers & Mod1Mask != 0 {
                            m |= Modifiers::LALT;
                        }
                        if modifiers & Mod4Mask != 0 {
                            m |= Modifiers::LSYS;
                        }
                        if modifiers & LockMask != 0 {
                            m |= Modifiers::CAPSLOCK;
                        }
                        if m.contains(w.modifiers) {
                            w.modifiers = m;
                            w.sender
                                .write()
                                .unwrap()
                                .send(WindowId(self.0), crate::WindowEvent::ModifiersChanged(m));
                        }
                    }
                    KeyRelease => {
                        let kr = unsafe { ev.key };
                        w.sender.write().unwrap().send(
                            WindowId(self.0),
                            crate::WindowEvent::KeyDown(crate::KeyboardInput {
                                key_code: kr.keycode as _,
                            }),
                        );

                        let modifiers =
                            kr.state & (ShiftMask | ControlMask | Mod1Mask | Mod4Mask | LockMask);
                        let mut m = Modifiers::empty();
                        if modifiers & ShiftMask != 0 {
                            m |= Modifiers::LSHIFT;
                        }
                        if modifiers & ControlMask != 0 {
                            m |= Modifiers::LCTRL;
                        }
                        if modifiers & Mod1Mask != 0 {
                            m |= Modifiers::LALT;
                        }
                        if modifiers & Mod4Mask != 0 {
                            m |= Modifiers::LSYS;
                        }
                        if modifiers & LockMask != 0 {
                            m |= Modifiers::CAPSLOCK;
                        }
                        if m.contains(w.modifiers) {
                            w.modifiers = m;
                            w.sender
                                .write()
                                .unwrap()
                                .send(WindowId(self.0), crate::WindowEvent::ModifiersChanged(m));
                        }
                    }
                    ButtonPress => {
                        let bp = unsafe { ev.button };
                        let button = match bp.button {
                            Button1 => MouseButtons::LCLICK,
                            Button2 => MouseButtons::RCLICK,
                            Button3 => MouseButtons::MCLICK,
                            Button4 => MouseButtons::BUTTON_4,
                            Button5 => MouseButtons::BUTTON_5,
                            _ => panic!(),
                        };
                        w.sender.write().unwrap().send(
                            WindowId(self.0),
                            crate::WindowEvent::MouseButtonDown(button),
                        );
                    }
                    ButtonRelease => {
                        let bp = unsafe { ev.button };
                        let button = match bp.button {
                            Button1 => MouseButtons::LCLICK,
                            Button2 => MouseButtons::RCLICK,
                            Button3 => MouseButtons::MCLICK,
                            Button4 => MouseButtons::BUTTON_4,
                            Button5 => MouseButtons::BUTTON_5,
                            _ => panic!(),
                        };
                        w.sender
                            .write()
                            .unwrap()
                            .send(WindowId(self.0), crate::WindowEvent::MouseButtonUp(button));
                    }
                    FocusIn => {
                        w.sender
                            .write()
                            .unwrap()
                            .send(WindowId(self.0), crate::WindowEvent::Focused(true));
                    }
                    FocusOut => {
                        w.sender
                            .write()
                            .unwrap()
                            .send(WindowId(self.0), crate::WindowEvent::Focused(false));
                    }
                    ClientMessage => {
                        let cm = unsafe { ev.client_message };
                        if cm.data.as_longs()[0]
                            == WM_DELETE_WINDOW.load(std::sync::atomic::Ordering::Relaxed) as _
                        {
                            unsafe { XDestroyWindow(w.display, self.0) };
                            unsafe { XCloseDisplay(w.display) };
                        }
                    }
                    _ => {}
                }
            })
            .or_insert(WindowInfo::default());
    }
}
