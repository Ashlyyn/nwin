#![allow(clippy::bool_comparison)]

use std::marker::PhantomData;

use bitflags::bitflags;

pub mod platform;

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

pub trait Window {
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
    fn set_title(&mut self, title: &str);
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
        self.fullscreen_type() == FullscreenType::Borderless || self.fullscreen_type() == FullscreenType::Exclusive
    }
    fn set_fullscreen(&mut self, fullscreen: FullscreenType);
    fn focus(&mut self);
    fn focused(&self) -> bool;
    fn request_user_attention(&mut self, attention: UserAttentionType);
    fn theme(&self) -> Theme;
    fn set_theme(&mut self, theme: Theme);
}

pub struct EventLoop {
    _no_send_sync: PhantomData<*mut ()>,
}
