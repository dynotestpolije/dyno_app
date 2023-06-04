pub mod about;
pub mod auth;
pub mod confirm_quit;
pub mod confirm_unsaved;
pub mod help;
pub mod logger;
pub mod save_server;
pub mod setting;

#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
pub use debug::*;

pub trait WindowState {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        state: &mut crate::state::DynoState,
    );
}

pub type WindowStack = Vec<Box<dyn WindowState>>;

pub fn window_states_new() -> Vec<Box<dyn WindowState>> {
    vec![
        Box::new(about::AboutWindow::new()),
        Box::new(auth::AuthWindow::new()),
        Box::new(confirm_quit::ConfirmQuitWindow::new()),
        Box::new(confirm_unsaved::ConfirmUnsavedWindow::new()),
        Box::new(help::HelpWindow::new()),
        Box::new(logger::LoggerWindow::new()),
        Box::new(save_server::SaveServerWindow::new()),
        Box::new(setting::SettingWindow::new()),
    ]
}
