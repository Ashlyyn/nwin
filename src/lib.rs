#![allow(clippy::bool_comparison, dead_code)]

use std::{
    collections::{HashSet, VecDeque},
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use bitflags::bitflags;

pub mod platform;

#[derive(Copy, Clone, Debug, Hash, Default, PartialEq, Eq)]
pub struct WindowId(pub u64);

bitflags! {
    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
    pub struct WindowButtons: u8 {
        const CLOSE = 0x00;
        const MINIMIZE = 0x01;
        const MAXIMIZE = 0x02;
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum WindowSizeState {
    Minimized,
    Maximized,
    #[default]
    Other,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FullscreenType {
    Exclusive,
    Borderless,
    #[default]
    NotFullscreen,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UserAttentionType {
    Critical,
    Informational,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

pub trait WindowT {
    fn id(&self) -> WindowId;
    fn request_redraw(&mut self);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);
    fn min_width(&self) -> u32;
    fn min_height(&self) -> u32;
    fn set_min_width(&mut self, width: u32);
    fn set_min_height(&mut self, height: u32);
    fn max_width(&self) -> u32;
    fn max_height(&self) -> u32;
    fn set_max_width(&mut self, width: u32);
    fn set_max_height(&mut self, height: u32);
    fn title(&self) -> String;
    fn visible(&self) -> bool;
    fn hide(&mut self);
    fn show(&mut self);
    fn resizeable(&self) -> bool;
    fn set_resizeable(&mut self, resizeable: bool);
    fn enabled_buttons(&self) -> WindowButtons;
    fn set_enabled_buttons(&mut self, buttons: WindowButtons);
    fn minimized(&self) -> bool;
    fn maximized(&self) -> bool;
    fn normalized(&self) -> bool;
    fn minimize(&mut self);
    fn maximize(&mut self);
    fn normalize(&mut self);
    fn fullscreen_type(&self) -> FullscreenType;
    fn fullscreen(&self) -> bool {
        self.fullscreen_type() == FullscreenType::Borderless
            || self.fullscreen_type() == FullscreenType::Exclusive
    }
    fn set_fullscreen(&mut self, fullscreen: FullscreenType);
    fn focus(&mut self);
    fn focused(&self) -> bool;
    fn request_user_attention(&mut self, attention: UserAttentionType);
    fn theme(&self) -> Theme;
    fn set_theme(&mut self, theme: Theme);
}

pub trait WindowTExt {
    fn sender(&self) -> Arc<RwLock<EventSender>>;
}

pub(crate) trait WindowIdExt {
    fn next_event(&self);
}

pub enum Window {
    //WindowsWindow(win32::Window),
}

#[derive(Copy, Clone, Default, Debug)]
pub struct KeyboardInput {
    key_code: u64,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    #[non_exhaustive]
    pub struct Modifiers: u16 {
        const LCTRL = 0x0001;
        const LSYS = 0x0002;
        const LALT = 0x0004;
        const LSHIFT = 0x0008;
        const RSHIFT = 0x0010;
        const RALT = 0x0020;
        const RSYS = 0x0040;
        const RCTRL = 0x0080;
        const CAPSLOCK = 0x0100;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    #[non_exhaustive]
    pub struct MouseButtons: u8 {
        const LCLICK = 0x01;
        const RCLICK = 0x02;
        const MCLICK = 0x04;
        const BUTTON_4 = 0x08;
        const BUTTON_5 = 0x10;
    }
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum WindowEvent {
    Resized(u32, u32),
    Moved(u32, u32),
    CloseRequested,
    Destroyed,
    Focused(bool),
    ThemeChanged(Theme),
    KeyDown(KeyboardInput),
    KeyUp(KeyboardInput),
    CursorMoved(f64, f64),
    MouseButtonDown(MouseButtons),
    MouseButtonUp(MouseButtons),
    ModifiersChanged(Modifiers),
    UnrecoverableError,
}

#[derive(Clone, Debug)]
pub struct EventSender {
    receiver: Option<Arc<RwLock<EventReceiver>>>,
}

impl EventSender {
    pub(crate) fn new() -> Self {
        Self { receiver: None }
    }

    pub(crate) fn with_receiver(receiver: Arc<RwLock<EventReceiver>>) -> Self {
        Self {
            receiver: Some(receiver),
        }
    }

    pub(crate) fn bind(&mut self, receiver: Arc<RwLock<EventReceiver>>) {
        self.receiver = Some(receiver);
    }

    pub(crate) fn send(&self, id: WindowId, ev: WindowEvent) {
        if let Some(r) = self.receiver.as_ref() {
            r.write().unwrap().recv(id, ev);
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventReceiver {
    events: VecDeque<(WindowId, WindowEvent)>, //_no_send: PhantomData<*mut ()>
}

impl EventReceiver {
    pub(crate) fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub(crate) fn recv(&mut self, id: WindowId, ev: WindowEvent) {
        dbg!(self.events.clone());
        self.events.push_back((id, ev));
    }
}

unsafe impl Sync for EventReceiver {}

#[derive(Debug)]
pub struct EventLoop {
    receiver: Arc<RwLock<EventReceiver>>,
    ids: HashSet<WindowId>,
    _no_send_sync: PhantomData<*mut ()>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            receiver: Arc::new(RwLock::new(EventReceiver::new())),
            ids: HashSet::new(),
            _no_send_sync: Default::default(),
        }
    }

    pub fn bind(&mut self, window: &mut (impl WindowT + WindowTExt)) {
        self.ids.insert(window.id());
        window.sender().write().unwrap().bind(self.receiver.clone());
    }

    pub fn next_event(&mut self) -> Option<(WindowId, WindowEvent)> {
        let events = {
            let receiver = self.receiver.read().unwrap();
            receiver.events.clone()
        };
        if events.is_empty() {
            for id in self.ids.clone() {
                id.next_event();
            }
        }
        let mut receiver = self.receiver.write().unwrap();
        receiver.events.pop_front()
    }

    pub(crate) fn events(&mut self) -> VecDeque<(WindowId, WindowEvent)> {
        let evs = self.receiver.write().unwrap().events.clone();
        self.receiver.write().unwrap().events.clear();
        evs
    }
}

pub(crate) trait EventLoopExt {
    fn cycle_event(&mut self);
}

mod tests {
    #[test]
    fn el_test() {}
}
