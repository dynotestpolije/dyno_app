use crate::widgets::{button::ButtonExt, DynoWidgets};
use dyno_types::{chrono::NaiveDateTime, data_buffer::Data};

#[derive(Debug, Default)]
pub struct DebugAction {
    rpm: f64,
    speed: f64,
    torque: f64,
    hp: f64,
    odo: f64,
    temp: f64,
    display_style: crate::widgets::DisplayStylePreset,
    start: bool,
}

impl DebugAction {
    pub fn get_preset(&self) -> crate::widgets::DisplayStylePreset {
        self.display_style
    }

    pub fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        buffer: &mut dyno_types::data_buffer::BufferData,
    ) {
        use dyno_types::convertions::prelude::*;
        let ctx_time = ctx.input(|i| i.time);
        let Self {
            rpm,
            speed,
            torque,
            hp,
            odo,
            temp,
            display_style,
            start,
        } = self;
        eframe::egui::Window::new("Debug Window")
            .id("window_debug_simulation".into())
            .show(ctx, |ui| {
                let hundread_euclid = ctx_time.rem_euclid(1.) * 100.;
                *rpm = ctx_time.rem_euclid(1.5) * 100.;
                *speed = ctx_time.rem_euclid(2.4) * 100.;
                *torque = hundread_euclid;
                *hp = hundread_euclid;
                *temp = hundread_euclid;
                *odo = ctx_time.rem_euclid(1.) * 0.01;
                eframe::egui::Grid::new("window_debug_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Start/Stop Emulation");
                        if ui.toggle(start).changed() {
                            dyno_types::log::debug!("Toggling start debug emulation");
                        }

                        if ui.reset_button().clicked() {
                            dyno_types::log::debug!("Resetting Buffer in debug emulation");
                            buffer.clean();
                        }

                        ui.end_row();

                        ui.label("RPM");
                        ui.label(format!("{:.2}", rpm));
                        ui.end_row();
                        ui.label("SPEED");
                        ui.label(format!("{:.2}", speed));
                        ui.end_row();
                        ui.label("TORQUE");
                        ui.label(format!("{:.2}", torque));
                        ui.end_row();
                        ui.label("HP");
                        ui.label(format!("{:.2}", hp));
                        ui.end_row();
                        ui.label("ODO");
                        ui.label(format!("{:.2}", odo));
                        ui.end_row();
                    });
                ui.separator();
                ui.combobox_from_iter(
                    "Style for SevenSegment",
                    display_style,
                    display_style.get_iter(),
                )
            });

        if *start {
            let data = Data {
                speed: KilometresPerHour::new(*speed),
                rpm: RotationPerMinute::new(*rpm),
                odo: KiloMetres::new(*odo),
                horsepower: *hp,
                torque: *torque,
                temp: Celcius::new(0.0),
                time_stamp: NaiveDateTime::MIN,
            };
            buffer.push_data(data);
        }
    }
}
