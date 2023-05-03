pub mod about;
pub mod confirm_quit;
pub mod confirm_unsaved;
pub mod help;
pub mod logger;
pub mod setting;

#[cfg(debug_assertions)]
mod debug;
#[cfg(debug_assertions)]
pub use debug::*;

pub trait WindowState {
    fn show_window(&mut self, ctx: &eframe::egui::Context, state: &mut crate::state::DynoState);
}

pub fn window_states_new(control: &crate::control::DynoControl) -> Vec<Box<dyn WindowState>> {
    vec![
        Box::new(about::AboutWindow::new()),
        Box::new(help::HelpWindow::new()),
        Box::new(logger::LoggerWindow::new()),
        Box::new(confirm_unsaved::ConfirmUnsavedWindow::new()),
        Box::new(confirm_quit::ConfirmQuitWindow::new()),
        Box::new(setting::SettingWindow::new(
            control.paths().clone(),
            control.config().clone(),
            control.info_motor().clone(),
        )),
    ]
}
