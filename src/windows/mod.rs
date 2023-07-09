pub mod about;
pub mod auth;
pub mod confirm_quit;
pub mod confirm_unsaved;
pub mod help;
pub mod logger;
pub mod open_server;
pub mod save_server;
pub mod setting;
pub mod stream_server;

pub use setting::PanelSetting;

#[cfg(debug_assertions)]
pub mod debug;

use downcast_rs::{impl_downcast, DowncastSync};

pub trait WindowState: DowncastSync {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        state: &mut crate::state::DynoState,
    );
    #[inline]
    fn set_open(&mut self, _open: bool) {}

    #[inline]
    fn is_open(&self) -> bool {
        false
    }
    #[inline]
    fn swap_open(&mut self) {
        self.set_open(!self.is_open())
    }
}
impl_downcast!(sync WindowState);

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WSIdx {
    About = 0,
    Auth,
    ConfirmQuit,
    ConfirmUnsaved,
    #[cfg(debug_assertions)]
    Debug,
    Help,
    Logger,
    OpenServer,
    SaveServer,
    StreamServer,
    Setting,
    WindowStateSize,
}
const WS_SIZE: usize = WSIdx::WindowStateSize as usize;

pub struct WindowStack {
    stack: [Box<dyn WindowState>; WS_SIZE],
}

impl Default for WindowStack {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowStack {
    pub fn new() -> Self {
        Self {
            stack: [
                Box::new(about::AboutWindow::new()),
                Box::new(auth::AuthWindow::new()),
                Box::new(confirm_quit::ConfirmQuitWindow::new()),
                Box::new(confirm_unsaved::ConfirmUnsavedWindow::new()),
                #[cfg(debug_assertions)]
                Box::<debug::DebugAction>::default(),
                Box::new(help::HelpWindow::new()),
                Box::new(logger::LoggerWindow::new()),
                Box::new(open_server::OpenServerWindow::new()),
                Box::new(save_server::SaveServerWindow::new()),
                Box::new(stream_server::StreamServerWindow::new()),
                Box::new(setting::SettingWindow::new()),
            ],
        }
    }

    #[inline]
    pub fn get(&self) -> &[Box<dyn WindowState>; WS_SIZE] {
        &self.stack
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut [Box<dyn WindowState>; WS_SIZE] {
        &mut self.stack
    }
    #[inline]
    pub fn idx<T: WindowState + 'static>(&self, idx: WSIdx) -> Option<&T> {
        self.stack[idx as usize].downcast_ref::<T>()
    }
    #[inline]
    pub fn idx_mut<T: WindowState + 'static>(&mut self, idx: WSIdx) -> Option<&mut T> {
        self.stack[idx as usize].downcast_mut::<T>()
    }
    #[inline]
    pub fn set_open(&mut self, idx: WSIdx, open: bool) {
        self.stack[idx as usize].set_open(open)
    }
    #[inline]
    pub fn set_swap_open(&mut self, idx: WSIdx) {
        self.stack[idx as usize].swap_open()
    }
}

impl core::ops::Deref for WindowStack {
    type Target = [Box<dyn WindowState>; WS_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}
impl core::ops::DerefMut for WindowStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stack
    }
}
