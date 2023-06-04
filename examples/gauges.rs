use dynotest_app::widgets::{Gauge, GaugeTypes};
use eframe::epaint::Vec2;

fn main() {
    let native_options = eframe::NativeOptions {
        transparent: false,
        maximized: false,
        fullscreen: false,
        always_on_top: true,
        initial_window_size: Some(Vec2 {
            x: 720_f32,
            y: 480_f32,
        }),
        resizable: false,
        ..Default::default()
    };
    eframe::run_native(
        "Image Viewer",
        native_options,
        Box::new(|cc| Box::new(ExampleGauge::new(cc))),
    )
    .expect("ERROR: Failed 'eframe::run_native'");
}

struct ExampleGauge {
    value: f32,
    presets: [GaugeTypes; 6],
}

impl ExampleGauge {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            value: 0f32,
            presets: [
                GaugeTypes::Default,
                GaugeTypes::RpmRodaGauge,
                GaugeTypes::RpmEngineGauge,
                GaugeTypes::SpeedGauge,
                GaugeTypes::TorqueGauge,
                GaugeTypes::HorsepowerGauge,
            ],
        }
    }
}

impl eframe::App for ExampleGauge {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            let width = (ui.available_size().x / 4.) - 6.;
            for preset in self.presets {
                Gauge::new(preset, self.value)
                    .animated(true)
                    .diameter(width);
            }
        });
    }
}
