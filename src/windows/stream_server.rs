use eframe::egui::{Button, Id, LayerId, RichText, Ui, Window};
use eframe::emath::Align2;
use eframe::epaint::{vec2, Color32, Rounding, Vec2};

#[derive(Debug, Clone, Default)]
pub struct StreamServerWindow {
    open: bool,
}
impl StreamServerWindow {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::WindowState for StreamServerWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
    ) {
        ctx.layer_painter(LayerId::new(
            eframe::egui::Order::Background,
            Id::new("confirmation_popup_unsaved"),
        ))
        .rect_filled(
            ctx.input(|inp| inp.screen_rect()),
            0.0,
            Color32::from_black_alpha(192),
        );

        let mut open = self.open;
        let ui_window = |ui: &mut Ui| {
            ui.heading("Info Dynotests: ");
            ui.add_space(10.);
            super::setting::SettingWindow::setting_info(ui, &mut control.config);
            ui.add_space(10.);
            let submit_btn = ui.add(
                Button::new(RichText::new("Stream").color(Color32::BLACK))
                    .rounding(Rounding::same(4.))
                    .fill(Color32::LIGHT_BLUE)
                    .min_size(vec2(280., 30.)),
            );

            if submit_btn.clicked() {
                open = false;
                control.service.start_stream();
                let config = control.config.clone();
                if let Some(api) = control.service.api() {
                    api.set_active(config);
                }
            }
        };

        Window::new("Stream to Server")
            .id("dyno_save_server".into())
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .open(&mut self.open)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| ui.vertical_centered_justified(ui_window));

        if !open {
            self.open = false;
        }
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
