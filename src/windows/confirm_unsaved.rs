use crate::widgets::button::ButtonExt;
use eframe::egui::{Align2, Vec2};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfirmUnsavedWindow {
    open: bool,
}
impl ConfirmUnsavedWindow {
    pub fn new() -> Self {
        Self::default()
    }
}
impl super::WindowState for ConfirmUnsavedWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
        state: &mut crate::state::DynoState,
    ) {
        if state.show_buffer_unsaved() {
            let painter = ctx.layer_painter(eframe::egui::LayerId::new(
                eframe::egui::Order::Background,
                eframe::egui::Id::new("confirmation_popup_unsaved"),
            ));
            painter.rect_filled(
                ctx.input(|inp| inp.screen_rect()),
                0.0,
                eframe::egui::Color32::from_black_alpha(192),
            );
        }

        eframe::egui::Window::new("Do you want to quit?")
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|horz_ui| {
                    if horz_ui.save_button().clicked() {
                        state.set_show_buffer_unsaved(false);
                        state.set_operator(crate::state::OperatorData::save_all());
                        state.set_quitable(true);
                    }
                    if horz_ui.no_button().clicked() {
                        state.set_show_buffer_unsaved(false);
                        state.set_quitable(true);
                    }
                    if horz_ui.cancel_button().clicked() {
                        state.set_show_buffer_unsaved(false);
                    }
                })
            });

        self.open = state.show_buffer_unsaved();
    }
}
