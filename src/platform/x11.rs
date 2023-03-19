#![allow(dead_code, non_upper_case_globals)]

use std::{ptr::addr_of_mut, mem::MaybeUninit, ffi::CString, sync::Arc};

use x11::xlib::{XOpenDisplay, XCreateWindow, InputOnly, InputOutput, CopyFromParent, Visual, XSetWindowAttributes, Pixmap, CWBackPixmap, CWBackPixel, CWBorderPixmap, CWBorderPixel, ForgetGravity, StaticGravity, NorthWestGravity, NorthGravity, NorthEastGravity, WestGravity, CenterGravity, EastGravity, SouthWestGravity, SouthGravity, SouthEastGravity, CWBitGravity, CWWinGravity, NotUseful, WhenMapped, Always, CWBackingStore, CWBackingPlanes, CWBackingPixel, CWSaveUnder, CWEventMask, CWDontPropagate, CWOverrideRedirect, Colormap, CWColormap, Cursor, CWCursor, PointerMotionMask, Button1MotionMask, Button2MotionMask, Button3MotionMask, Button4MotionMask, Button5MotionMask, ButtonMotionMask, KeyPressMask, KeyReleaseMask, ButtonPressMask, ButtonReleaseMask, EnterWindowMask, LeaveWindowMask, PointerMotionHintMask, KeymapStateMask, ExposureMask, VisibilityChangeMask, StructureNotifyMask, ResizeRedirectMask, SubstructureNotifyMask, SubstructureRedirectMask, FocusChangeMask, PropertyChangeMask, ColormapChangeMask, OwnerGrabButtonMask, XSelectInput, XMapWindow, XStoreName, XRootWindow, XDefaultScreen, XSetWMNormalHints, XAllocSizeHints, PMinSize, PMaxSize, XUnmapWindow, XIconifyWindow, XClientMessageEvent, ClientMessage, XInternAtom, ClientMessageData, XSendEvent, XDefaultRootWindow, XSetInputFocus, RevertToParent, CurrentTime, XRaiseWindow, XResizeWindow, XDestroyWindow};

use crate::{WindowButtons, FullscreenType, WindowId, WindowSizeState, Theme};

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
enum Gravity {
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
enum BackingStore {
    NotUseful = NotUseful,
    WhenMapped = WhenMapped,
    Always = Always,
}

impl BackingStore {
    pub fn as_i32(&self) -> i32 {
        *self as _
    }
}

struct BackingPlanes(u64);

bitflags::bitflags! {
    #[derive(Copy, Clone, Default, Debug)]
    struct EventMask: i64 {
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
struct WindowAttributes {
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

struct WindowAttributesBuilder {
    inner: WindowAttributes,
}

impl WindowAttributesBuilder {
    pub fn new() -> Self {
        Self { 
            inner: WindowAttributes { 
                inner: unsafe {
                    MaybeUninit::zeroed().assume_init()
                }, 
                mask: 0 
            } 
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
    mut visual: Option<Visual>,
    attributes: Option<WindowAttributes>,
    event_mask: EventMask,
) -> Result<(x11::xlib::Window, *mut x11::xlib::Display), ()> {
    let visual = if let Some(ref mut v) = visual { addr_of_mut!(*v) } else { core::ptr::null_mut() };
    let mask = if let Some(ref a) = attributes { a.mask } else { 0 };
    let attributes = if let Some(mut a) = attributes { addr_of_mut!(a.inner) } else { core::ptr::null_mut() };

    let display = unsafe { XOpenDisplay(core::ptr::null()) };
    if display.is_null() {
        return Err(());
    }

    let window = unsafe { XCreateWindow(
        display, 
        parent.unwrap_or_else(|| {
            XRootWindow(display,  XDefaultScreen(display))
        }), 
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
    ) };
    assert_ne!(window, 0);

    if window < 16 {
        return Err(())
    }

    unsafe { XSelectInput(display, window, event_mask.bits()) };
    if visible { unsafe { XMapWindow(display, window); } };
    let window_name_c = CString::new(window_name).unwrap();
    unsafe { XStoreName(display, window, window_name_c.as_ptr()) };
    Ok((window, display))
}

mod tests {
    //#[test]
    fn cw_test() {
        use std::{mem::MaybeUninit, ptr::addr_of_mut};
        use x11::xlib::{XEvent, XNextEvent, KeyPress};
        use super::{create_window, WindowClass, EventMask};
        use x11::xlib::{XDestroyWindow};

        let (id, display) = create_window(
            "test window", None, 0, 0, 600, 400, true, 10, 
            None, WindowClass::InputOutput, 
            None, None, EventMask::all()
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

        let (id, display) = create_window(
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
        use crate::platform::x11::{WindowExtXlib, EventMask};
        use x11::xlib::{FocusIn, FocusOut, MapNotify, UnmapNotify, ReparentNotify, ConfigureNotify, ResizeRequest};
        use crate::Window;

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
}

#[derive(Clone, Debug)]
pub(crate) struct Window {
    name: String,
    display: *mut x11::xlib::Display,
    //screen: *mut x11::xlib::Screen,
    screen_number: i32,
    id: Arc<x11::xlib::Window>,
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
}

impl Default for Window {
    fn default() -> Self {
        Self {
            name: "nwin window".to_owned(),
            display: core::ptr::null_mut(),
            id: Arc::new(0),
            parent: 0,
            screen_number: 0,
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
            event_mask: EventMask::empty(),
            enabled_buttons: WindowButtons::all(),
            focused: false,
            fullscreen: FullscreenType::NotFullscreen,
            size_state: WindowSizeState::Other,
            resizeable: false,
            theme: Theme::Light,
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if Arc::strong_count(&self.id) <= 1 {
            unsafe { XDestroyWindow(self.display, *self.id) };
        }
    }
}

impl Window {
    fn try_new(parent: Option<x11::xlib::Window>, attributes: Option<WindowAttributes>) -> Result<Self, ()> {
        let mut s = Self::default();
        let (id, display) = s.create(parent, attributes)?;
        s.id = Arc::new(id);
        s.display = display;
        s.screen_number = unsafe { XDefaultScreen(display) };
        s.parent = parent.unwrap_or(unsafe { XRootWindow(display, s.screen_number) });
        Ok(s)
    }

    fn create(&self, parent: Option<x11::xlib::Window>, attributes: Option<WindowAttributes>) -> Result<(x11::xlib::Window, *mut x11::xlib::Display), ()> {
        create_window(
            &self.name, 
            parent, 
            self.x, 
            self.y, 
            self.width, 
            self.height, 
            self.visible,
            self.border_width, 
            Some(self.depth), 
            self.class, 
            self.visual, 
            attributes, 
            self.event_mask
        )
    }
}

impl crate::Window for Window {
    fn enabled_buttons(&self) -> crate::WindowButtons {
        self.enabled_buttons
    }

    fn set_enabled_buttons(&mut self, buttons: WindowButtons) {
        /*
        let allowed_actions_s = CString::new("_NET_WM_ALLOWED_ACTIONS").unwrap();
        let maximize_horz_s = CString::new("_NET_WM_ACTION_MAXIMIZE_HORZ").unwrap();
        let maximize_vert_s = CString::new("_NET_WM_ACTION_MAXIMIZE_VERT").unwrap();

        let allowed_actions = unsafe { XInternAtom(self.display, allowed_actions_s.as_ptr(), x11::xlib::False) };
        let maximize_horz = unsafe { XInternAtom(self.display, maximize_horz_s.as_ptr(), x11::xlib::False) };
        let maximize_vert = unsafe { XInternAtom(self.display, maximize_vert_s.as_ptr(), x11::xlib::False) };

        unsafe { XChangeProperty(self.display, *self.id, allowed_actions, XA_ATOM, 32, PropModeAppend, addr_of_mut!(maximize_horz) as _, 1) }
        */
        if buttons != WindowButtons::all() {
            todo!()
        }
    }

    fn focus(&mut self) {
        self.focused = true;
        unsafe { XSetInputFocus(self.display, *self.id, RevertToParent, CurrentTime) };
        unsafe { XRaiseWindow(self.display, *self.id) };
    }
    
    fn focused(&self) -> bool {
        self.focused
    }

    fn fullscreen_type(&self) -> FullscreenType {
        self.fullscreen
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn set_width(&mut self, width: u32) {
        self.width = width;
        unsafe { XResizeWindow(self.display, *self.id, self.width, self.height) };
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn set_height(&mut self, height: u32) {
        self.height = height;
        unsafe { XResizeWindow(self.display, *self.id, self.width, self.height) };
    }

    fn id(&self) -> WindowId {
        WindowId(*self.id as _)
    }

    fn min_width(&self) -> u32 {
        self.min_width
    }

    fn set_min_width(&mut self, width: u32) {
        self.min_width = width;
        let size_hints = &mut unsafe { *XAllocSizeHints() };
        size_hints.min_width = self.min_width as _;
        size_hints.min_height = self.min_height as _;
        size_hints.flags = PMinSize;
        unsafe { XSetWMNormalHints(self.display, *self.id, addr_of_mut!(*size_hints)) };
        unsafe { libc::free(addr_of_mut!(*size_hints) as _) };
    }

    fn min_height(&self) -> u32 {
        self.min_height
    }

    fn set_min_height(&mut self, height: u32) {
        self.min_height = height;
        let size_hints = &mut unsafe { *XAllocSizeHints() };
        size_hints.min_width = self.min_width as _;
        size_hints.min_height = self.min_height as _;
        size_hints.flags = PMinSize;
        unsafe { XSetWMNormalHints(self.display, *self.id, addr_of_mut!(*size_hints)) };
        unsafe { libc::free(addr_of_mut!(*size_hints) as _) };
    }

    fn max_width(&self) -> u32 {
        self.max_width
    }

    fn set_max_width(&mut self, width: u32) {
        self.max_width = width;
        let size_hints = &mut unsafe { *XAllocSizeHints() };
        size_hints.max_width = self.max_width as _;
        size_hints.max_height = self.max_height as _;
        size_hints.flags = PMaxSize;
        unsafe { XSetWMNormalHints(self.display, *self.id, addr_of_mut!(*size_hints)) };
        unsafe { libc::free(addr_of_mut!(*size_hints) as _) };
    }

    fn max_height(&self) -> u32 {
        self.max_height
    }

    fn set_max_height(&mut self, height: u32) {
        self.max_height = height;
        let size_hints = &mut unsafe { *XAllocSizeHints() };
        size_hints.max_width = self.max_width as _;
        size_hints.max_height = self.max_height as _;
        size_hints.flags = PMaxSize;
        unsafe { XSetWMNormalHints(self.display, *self.id, addr_of_mut!(*size_hints)) };
        unsafe { libc::free(addr_of_mut!(*size_hints) as _) };
    }

    fn maximized(&self) -> bool {
        self.size_state == WindowSizeState::Maximized
    }

    fn maximize(&mut self) {
        const NET_WM_TOGGLE_STATE: i64 = 2;

        let wm_state_s = CString::new("_NET_WM_STATE").unwrap();
        let max_width_s = CString::new("_NET_WM_STATE_MAXIMIZED_HORZ").unwrap();
        let max_height_s = CString::new("_NET_WM_STATE_MAXIMIZED_VERT").unwrap();
        
        let wm_state = unsafe { XInternAtom(self.display, wm_state_s.as_ptr(), x11::xlib::False) };
        let max_width = unsafe { XInternAtom(self.display, max_width_s.as_ptr(), x11::xlib::False) };
        let max_height = unsafe { XInternAtom(self.display, max_height_s.as_ptr(), x11::xlib::False) };

        let mut ev = XClientMessageEvent {
            type_: ClientMessage,
            format: 32,
            window: *self.id,
            message_type: wm_state,
            data: ClientMessageData::from([ NET_WM_TOGGLE_STATE, max_width as _, max_height as _, 1, 0 ]),
            serial: 0,
            send_event: 0,
            display: self.display
        };

        unsafe { XSendEvent(self.display, XDefaultRootWindow(self.display), x11::xlib::False, SubstructureNotifyMask, addr_of_mut!(ev) as _) };
        self.size_state = WindowSizeState::Maximized;
    }

    fn minimized(&self) -> bool {
        self.size_state == WindowSizeState::Minimized
    }

    fn minimize(&mut self) {
        unsafe { XIconifyWindow(self.display, *self.id, self.screen_number) };
        self.size_state = WindowSizeState::Minimized;
    }

    fn normalized(&self) -> bool {
        self.size_state == WindowSizeState::Other
    }

    // TODO - implement better
    fn normalize(&mut self) {
        if self.maximized() {
            self.maximize();
        } else {
            self.maximize();
            self.maximize();
        }

        self.size_state = WindowSizeState::Other;
    }

    fn resizeable(&self) -> bool {
        self.resizeable
    }

    fn set_resizeable(&mut self, resizeable: bool) {
        self.resizeable = resizeable;
        let size_hints = &mut unsafe { *XAllocSizeHints() };
        if resizeable == false {
            size_hints.min_width = self.width as _;
            size_hints.max_width = self.width as _;
            size_hints.min_height = self.height as _;
            size_hints.max_height = self.height as _;
        } else {
            size_hints.min_width = self.min_width as _;
            size_hints.max_width = self.max_width as _;
            size_hints.min_height = self.min_height as _;
            size_hints.max_height = self.min_height as _;
        }
        size_hints.flags = PMinSize | PMaxSize;
        unsafe { XSetWMNormalHints(self.display, *self.id, addr_of_mut!(*size_hints)) };
    }

    fn theme(&self) -> Theme {
        self.theme
    }

    fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        todo!()
    }

    fn title(&self) -> String {
        self.name.clone()
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn hide(&mut self) {
        unsafe { XUnmapWindow(self.display, *self.id) };
    }

    fn show(&mut self) {
        unsafe { XMapWindow(self.display, *self.id) };
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
        self.event_mask
    }

    fn set_event_mask(&mut self, event_mask: EventMask) {
        self.event_mask = event_mask;
        unsafe { XSelectInput(self.display, *self.id, event_mask.bits()) };
    }

    fn set_title(&mut self, title: &str) {
        let title_c = CString::new(title).unwrap();
        unsafe { XStoreName(self.display, *self.id, title_c.as_ptr()) };
    }
}