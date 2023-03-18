#![allow(dead_code, non_upper_case_globals)]

use std::{ptr::addr_of_mut, mem::MaybeUninit, ffi::CString};

use x11::xlib::{XOpenDisplay, XCreateWindow, InputOnly, InputOutput, CopyFromParent, Visual, XSetWindowAttributes, Pixmap, CWBackPixmap, CWBackPixel, CWBorderPixmap, CWBorderPixel, ForgetGravity, StaticGravity, NorthWestGravity, NorthGravity, NorthEastGravity, WestGravity, CenterGravity, EastGravity, SouthWestGravity, SouthGravity, SouthEastGravity, CWBitGravity, CWWinGravity, NotUseful, WhenMapped, Always, CWBackingStore, CWBackingPlanes, CWBackingPixel, CWSaveUnder, CWEventMask, CWDontPropagate, CWOverrideRedirect, Colormap, CWColormap, Cursor, CWCursor, PointerMotionMask, Button1MotionMask, Button2MotionMask, Button3MotionMask, Button4MotionMask, Button5MotionMask, ButtonMotionMask, KeyPressMask, KeyReleaseMask, ButtonPressMask, ButtonReleaseMask, EnterWindowMask, LeaveWindowMask, PointerMotionHintMask, KeymapStateMask, ExposureMask, VisibilityChangeMask, StructureNotifyMask, ResizeRedirectMask, SubstructureNotifyMask, SubstructureRedirectMask, FocusChangeMask, PropertyChangeMask, ColormapChangeMask, OwnerGrabButtonMask, XSelectInput, XMapWindow, XStoreName, XRootWindow, XDefaultScreen};

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
        depth.unwrap_or(0), 
        class.as_u32(), 
        visual, 
        mask, 
        attributes,
    ) };

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
    #[test]
    fn cw_test() {
        use std::{mem::MaybeUninit, ptr::addr_of_mut};
        use x11::xlib::{XEvent, XNextEvent, KeyPress};
        use super::{create_window, WindowClass, EventMask};

        let (_window, display) = create_window(
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
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Window {
    name: String,
    id: x11::xlib::Window,
    parent: x11::xlib::Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    visible: bool,
    border_width: u32,
    depth: i32,
    class: WindowClass,
    visual: Option<Visual>,
    attributes: WindowAttributes,
    event_mask: EventMask,
}

impl Window {
    fn create(&self) -> Result<(x11::xlib::Window, *mut x11::xlib::Display), ()> {
        create_window(
            &self.name, 
            Some(self.parent), 
            self.x, 
            self.y, 
            self.width, 
            self.height, 
            self.visible,
            self.border_width, 
            Some(self.depth), 
            self.class, 
            self.visual, 
            Some(self.attributes), 
            self.event_mask
        )
    }
}