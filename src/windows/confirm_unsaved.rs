use crate::widgets::button::{ButtonExt, ButtonKind};
use eframe::egui::{
    Align2, Color32, Context, Id, InnerResponse, Key, LayerId, Order, Vec2, Window,
};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConfirmUnsavedWindow;
impl ConfirmUnsavedWindow {
    pub fn new() -> Self {
        Self
    }
}
impl super::WindowState for ConfirmUnsavedWindow {
    fn show_window(&mut self, ctx: &Context, state: &mut crate::state::DynoState) {
        if state.show_buffer_unsaved() {
            let painter = ctx.layer_painter(LayerId::new(
                Order::Background,
                Id::new("confirmation_popup_unsaved"),
            ));
            painter.rect_filled(
                ctx.input(|inp| inp.screen_rect()),
                0.0,
                Color32::from_black_alpha(192),
            );
        }

        match Window::new("Buffer Data Records is unsaved. Do you want to save it?")
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .open(state.show_buffer_unsaved_mut())
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("there is data recorded in buffer, and its not saved");
                ui.small("click 'Save' to save it or 'No' to rebort it, or 'Cancel' to cancel it");
                ui.horizontal(|horz_ui| {
                    if horz_ui.save_button().clicked() || horz_ui.input(|i| i.key_down(Key::Enter))
                    {
                        Some(ButtonKind::Save)
                    } else if horz_ui.no_button().clicked() {
                        Some(ButtonKind::No)
                    } else if horz_ui.cancel_button().clicked()
                        || horz_ui.input(|i| i.key_down(Key::Escape))
                    {
                        Some(ButtonKind::Cancel)
                    } else {
                        None
                    }
                })
                .inner
            }) {
            Some(InnerResponse {
                inner: Some(Some(ButtonKind::Save)),
                ..
            }) => {
                state.set_operator(crate::state::OperatorData::save_all());
                state.set_show_buffer_unsaved(false);
                state.set_show_quitable(true);
            }
            Some(InnerResponse {
                inner: Some(Some(ButtonKind::No)),
                ..
            }) => {
                state.set_operator(crate::state::OperatorData::Noop);
                state.set_show_buffer_unsaved(false);
                state.set_show_quitable(true);
            }
            Some(InnerResponse {
                inner: Some(Some(ButtonKind::Cancel)),
                ..
            }) => state.set_show_buffer_unsaved(false),
            _ => {}
        }
    }
}
