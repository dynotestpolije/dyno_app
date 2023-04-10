use dynotest_app::widgets::{
    frame_history::FrameHistory,
    gauges::{GaugeTypes, Gauges},
    *,
};
use eframe::{
    egui::{Layout, Slider},
    epaint::{Color32, Vec2},
};

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
    );
}

struct ExampleGauge {
    value: f32,
    presets: [GaugeTypes; 5],
    frame: FrameHistory,
}

impl ExampleGauge {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            value: 0f32,
            frame: FrameHistory::default(),
            presets: [
                GaugeTypes::Default,
                GaugeTypes::RpmGauge,
                GaugeTypes::SpeedGauge,
                GaugeTypes::TorqueGauge,
                GaugeTypes::HorsepowerGauge,
            ],
        }
    }
}

impl eframe::App for ExampleGauge {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.frame
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            let changed = ui
                .with_layout(Layout::top_down(eframe::emath::Align::Min), |ui| {
                    ui.heading("Egui Example");
                    ui.separator();
                    self.frame.ui(ui);
                    ui.label(format!(
                        "Repainting the UI each frame. FPS: {:.1}",
                        self.frame.fps()
                    ));
                    ui.separator();
                    ui.add(Slider::new(&mut self.value, 0f32..=200f32).step_by(0.1))
                        .is_pointer_button_down_on()
                })
                .inner;
            ui.separator();
            ui.with_layout(
                Layout::left_to_right(eframe::emath::Align::Min).with_main_wrap(true),
                |ui| {
                    let width = (ui.available_size().x / 4.) - 6.;
                    for preset in self.presets {
                        Gauges::new(preset, self.value)
                            .animated(true)
                            .diameter(width)
                    }
                },
            );
        });
    }
}
