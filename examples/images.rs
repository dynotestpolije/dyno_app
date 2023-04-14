use dynotest_app::widgets::images::Img;

fn main() {
    let native_options = eframe::NativeOptions {
        transparent: false,
        maximized: false,
        fullscreen: false,
        always_on_top: true,
        initial_window_size: Some(eframe::egui::Vec2 {
            x: 720_f32,
            y: 480_f32,
        }),
        resizable: false,
        ..Default::default()
    };
    eframe::run_native("Image Viewer", native_options, Box::new(ExampleImages::new))
        .expect("ERROR: Failed 'eframe::run_native'");
}

struct ExampleImages {
    images_rester: Img,
}
const IMAGE_BYTES: &[u8] = include_bytes!("./example.png");

impl ExampleImages {
    #[allow(clippy::new_ret_no_self)]
    fn new(_cc: &eframe::CreationContext) -> Box<dyn eframe::App> {
        let Ok(images_rester) = Img::from_image_bytes(stringify!(IMAGE_BYTES), IMAGE_BYTES) else {
            eprintln!("ERROR: failed to load image bytes in ExampleImages");
            std::process::exit(1);
        };
        Box::new(Self { images_rester })
    }
}

impl eframe::App for ExampleImages {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                eframe::egui::Layout::left_to_right(eframe::egui::Align::Center),
                |ui| {
                    let size = eframe::egui::Vec2::splat(ui.available_size().max_elem() * 0.4);
                    self.images_rester.show_size(ui, size);
                },
            );
        });
    }
}
