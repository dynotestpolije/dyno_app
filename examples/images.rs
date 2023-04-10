use eframe::{egui::Layout, epaint::Vec2, App, CreationContext};
use widgets::{
    frame_history,
    image::{load_image_bytes, load_svg_bytes, Images},
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
    eframe::run_native("Image Viewer", native_options, Box::new(ExampleGauge::new));
}

struct ExampleGauge {
    images_rester: Images,
    images_svg: Images,
    frame: frame_history::FrameHistory,
}
const IMAGE_BYTES: &'static [u8] = include_bytes!("./example.png");
const IMAGE_SVG: &'static [u8] = include_bytes!("./example.svg");

impl ExampleGauge {
    fn new(_cc: &CreationContext) -> Box<dyn App> {
        let colorimage_rester = load_image_bytes(IMAGE_BYTES).unwrap();
        let images_rester = Images::from_color_image(colorimage_rester);
        let colorimage_svg = load_svg_bytes(IMAGE_SVG).unwrap();
        let images_svg = Images::from_color_image(colorimage_svg);
        Box::new(Self {
            images_rester,
            images_svg,
            frame: frame_history::FrameHistory::default(),
        })
    }
}

impl App for ExampleGauge {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.frame
            .on_new_frame(ctx.input().time, frame.info().cpu_usage);
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(eframe::emath::Align::Min), |ui| {
                ui.heading("Egui Example");
                ui.separator();
                self.frame.ui(ui);
                ui.label(format!(
                    "Repainting the UI each frame. FPS: {:.1}",
                    self.frame.fps()
                ));
                ui.separator();
            });

            ui.separator();
            ui.with_layout(Layout::left_to_right(eframe::egui::Align::Center), |ui| {
                let size = Vec2::splat(ui.available_size().max_elem() * 0.4);
                self.images_rester.show_size("img-example-rester", ui, size);
                ui.separator();
                let size = Vec2::splat(ui.available_size().max_elem() * 0.4);
                self.images_svg.show_size("img-example-svg", ui, size);
            });
        });
    }
}
