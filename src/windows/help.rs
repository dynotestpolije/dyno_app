use dyno_core::serde;
use eframe::egui::Window;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(crate = "serde")]
pub struct HelpWindow {
    open: bool,
}
impl HelpWindow {
    pub fn new() -> Self {
        Self::default()
    }
}

// TODO(rizal_achp): implement help window
impl super::WindowState for HelpWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        _control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
    ) {
        Window::new("Help")
            .open(&mut self.open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui: &mut eframe::egui::Ui| {
                ui.heading("Upcoming Update! Help Window")
            });
    }

    #[inline]
    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.open
    }
}
