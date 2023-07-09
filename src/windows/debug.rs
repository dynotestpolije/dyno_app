use crate::widgets::{button::ButtonExt, DynoWidgets};
use dyno_core::{chrono::Utc, convertions, data_structure::ExponentialFilter, Data};

#[derive(Debug, Default)]
pub struct DebugAction {
    rpm: f64,
    speed: f64,
    torque: f64,
    hp: f64,
    odo: f64,
    temp: f64,
    speed_filter: ExponentialFilter<f64>,
    display_style: crate::widgets::DisplayStylePreset,
    start: bool,
}
impl super::WindowState for DebugAction {
    fn set_open(&mut self, _open: bool) {}

    fn is_open(&self) -> bool {
        true
    }

    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
    ) {
        use convertions::prelude::*;
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
            speed_filter,
        } = self;
        eframe::egui::Window::new("Debug Window")
            .id("window_debug_simulation".into())
            .show(ctx, |ui| {
                let hundread_euclid = ctx_time.rem_euclid(1.) * 100.;
                *rpm = ctx_time.rem_euclid(15.) * 1000.;
                *speed = ctx_time.rem_euclid(24.) * 10.;
                *torque = hundread_euclid;
                *hp = hundread_euclid;
                *temp = hundread_euclid;
                *odo += ctx_time.rem_euclid(1.) * 0.01;
                eframe::egui::Grid::new("window_debug_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Start/Stop Emulation");
                        if ui.toggle(start).changed() {
                            crate::log::debug!("Toggling start debug emulation");
                        }

                        if ui.reset_button().clicked() {
                            crate::log::debug!("Resetting Buffer in debug emulation");
                            control.buffer.clean();
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

        if *start && (((ctx_time as u64).rem_euclid(1) * 1000) % 250) == 0 {
            let rpm = RotationPerMinute::new(*rpm);
            let data = Data {
                speed: KilometresPerHour::new(speed_filter.next(*speed)),
                rpm_roda: rpm,
                rpm_engine: rpm,
                odo: KiloMetres::new(*odo),
                horsepower: HorsePower::new(*hp),
                torque: NewtonMeter::new(*torque),
                temp: Celcius::new(0.0),
                time_stamp: Utc::now().naive_utc(),
                ..Default::default()
            };
            control.service.send_stream_data(&data);
            control.buffer.push_from_data(&mut control.config, data);
        }
    }
}
