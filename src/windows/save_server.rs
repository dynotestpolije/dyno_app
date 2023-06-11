use dyno_core::{Cylinder, MotorType, Stroke as InfoMotorStroke, Transmition};
use eframe::egui::{Button, DragValue, Id, LayerId, RichText, TextEdit, Ui, Window};
use eframe::emath::Align2;
use eframe::epaint::{vec2, Color32, Rounding, Vec2};

use crate::toast_warn;
use crate::widgets::DynoWidgets;

#[derive(Debug, Clone, Default)]
pub struct SaveServerWindow {
    open: bool,
}
impl SaveServerWindow {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::WindowState for SaveServerWindow {
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

        let ui_window = |ui: &mut Ui| {
            ui.heading("The Info Dynotests: ");
            ui.add_space(10.);
            match &mut control.config.motor_type {
                MotorType::Electric(_) => todo!(),
                MotorType::Engine(ref mut info) => {
                    ui.separator();
                    ui.horizontal_wrapped(|horzui| {
                        horzui
                            .add(TextEdit::singleline(&mut info.name).hint_text("isi nama motor"));
                        horzui.separator();
                        horzui.add(
                            DragValue::new(&mut info.cc)
                                .speed(1)
                                .prefix("Volume Cilinder: ")
                                .suffix(" cc")
                                .min_decimals(10)
                                .max_decimals(30),
                        );
                    });
                    ui.separator();
                    ui.horizontal_wrapped(|horzui| {
                        horzui
                            .selectable_value_from_iter(&mut info.cylinder, Cylinder::into_iter());
                        horzui.separator();
                        horzui.selectable_value_from_iter(
                            &mut info.stroke,
                            InfoMotorStroke::into_iter(),
                        );
                        horzui.separator();
                        horzui.selectable_value_from_iter(
                            &mut info.transmition,
                            Transmition::into_iter(),
                        );
                    });
                }
            };

            let submit_btn = ui.add(
                Button::new(RichText::new("Save").color(Color32::BLACK))
                    .rounding(Rounding::same(4.))
                    .fill(Color32::LIGHT_BLUE)
                    .min_size(vec2(280., 30.)),
            );

            if submit_btn.clicked() {
                match control.api() {
                    Some(api) => {
                        let buffer = control.buffer().clone();
                        let config = control.config.clone();
                        let start = control.start.unwrap_or_default();
                        let stop = control.stop.unwrap_or_default();
                        api.save_dyno(buffer, config, start, stop, control.tx().clone());
                    }
                    None => {
                        toast_warn!("Not connected to API, try reconnecting or check the internet connection.")
                    }
                }
            }
        };

        Window::new("Save DynoTests to Server")
            .id("dyno_save_server".into())
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .open(&mut self.open)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| ui.vertical_centered_justified(ui_window));
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
