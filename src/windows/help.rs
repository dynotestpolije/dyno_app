#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HelpWindow;
impl HelpWindow {
    pub fn new() -> Self {
        Self
    }
}

// TODO(rizal_achp): implement help window
impl super::WindowState for HelpWindow {
    fn show_window(
        &mut self,
        _ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
        state: &mut crate::state::DynoState,
    ) {
        if !state.show_help() {
            return;
        }
        todo!("implement `show_help` window")
    }
}
