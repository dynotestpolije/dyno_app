pub mod about;
pub mod auth;
pub mod confirm_quit;
pub mod confirm_unsaved;
pub mod help;
pub mod logger;
pub mod open_server;
pub mod save_server;
pub mod setting;

#[cfg(debug_assertions)]
pub mod debug;

pub trait WindowState {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        state: &mut crate::state::DynoState,
    );
    #[inline]
    fn set_open(&mut self, open: bool) {
        unimplemented!("{open:#?}")
    }
    #[inline]
    fn is_open(&self) -> bool {
        false
    }
    #[inline]
    fn swap_open(&mut self) {
        self.set_open(!self.is_open())
    }
}

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
    pub fn idx(&self, idx: WSIdx) -> &dyn WindowState {
        self.stack[idx as usize].as_ref()
    }
    #[inline]
    pub fn idx_mut(&mut self, idx: WSIdx) -> &mut dyn WindowState {
        self.stack[idx as usize].as_mut()
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
