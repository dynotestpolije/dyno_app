#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use dyno_core::{ignore_err, log, serde, tokio};
use dynotest_app::{
    config::ApplicationConfig, control::DynoControl, init_logger, msg_dialog_err, paths::DynoPaths,
    state::DynoState, widgets::RealtimePlot, windows::WindowStack, PanelId, APP_KEY, PACKAGE_INFO,
    TOAST_MSG,
};
use eframe::egui::*;

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
#[serde(default)]
pub struct Applications {
    #[serde(skip)]
    #[serde(default = "DynoControl::new")]
    control: DynoControl,
    #[serde(skip)]
    #[serde(default = "WindowStack::new")]
    window_stack: WindowStack,

    plots: RealtimePlot,
    state: DynoState,
}

impl Applications {
    #[allow(clippy::new_ret_no_self)]
    pub fn run(paths: DynoPaths) {
        let opt = paths
            .get_config::<ApplicationConfig>("app_config.toml")
            .unwrap_or_else(|err| {
                dyno_core::log::error!("{err}");
                Default::default()
            });
        let app_creator: eframe::AppCreator = Box::new(|cc| {
            log::debug!("IntegrationInfo: {:?}", cc.integration_info);
            Box::new(
                cc.storage
                    .and_then(|s| eframe::get_value::<Self>(s, APP_KEY))
                    .unwrap_or_default()
                    .init(),
            )
        });

        if let Err(err) = eframe::run_native(
            PACKAGE_INFO.app_name,
            opt.app_options.main_window_opt(),
            app_creator,
        ) {
            dyno_core::log::error!("Failed to run app eframe in native - {err}");
            if !msg_dialog_err!(
                OkReport => ["Ignore the Error and close the Application", "Report the error to Developer"],
                "ERROR Running Applications",
                "Failed to running the aplication because: {err}"
            ) {
                dyno_core::log::warn!("ERROR: TODO! report the error from run native");
            }
        }
    }
    pub fn init(mut self) -> Self {
        self.control.init();
        self
    }
    pub fn deinit(&mut self) {
        self.control.deinit();
    }
}

impl Applications {
    fn main_panels_draw(&mut self, ctx: &Context) {
        let width = ctx.available_rect().width();
        let height = ctx.available_rect().height();

        TopBottomPanel::top(PanelId::Top).show(ctx, |ui| {
            menu::bar(ui, |uibar| {
                use dynotest_app::assets::POLIJE_LOGO_PNG as IMG;
                uibar.image(IMG.texture_id(uibar.ctx()), IMG.size_vec2());
                uibar.heading("DynoTests Polije");
                uibar.separator();
                self.control
                    .top_panel(uibar, &mut self.window_stack, &mut self.state);
            })
        });

        TopBottomPanel::bottom(PanelId::Bottom)
            .max_height(height * 0.05)
            .show_animated(ctx, self.state.show_bottom_panel(), |ui| {
                self.control.bottom_status(ui, &mut self.window_stack)
            });

        SidePanel::left(PanelId::Left)
            .min_width(width * 0.45)
            .max_width(width * 0.5)
            .show_animated(ctx, self.state.show_left_panel(), |ui| {
                self.control.left_panel(ui)
            });
        CentralPanel::default().show(ctx, |ui| {
            self.control.right_panel(ui);
            ui.separator();
            self.plots.ui(ui, &self.control.buffer);
        });
    }
}

impl eframe::App for Applications {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        for window in self.window_stack.iter_mut() {
            if window.is_open() {
                window.show_window(ctx, &mut self.control, &mut self.state)
            }
        }
        crate::TOAST_MSG.lock().show(ctx);

        self.control.handle_states(ctx);
        self.main_panels_draw(ctx);

        if self.state.quit() {
            frame.close();
        }
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        self.control
            .on_pos_render(&mut self.window_stack, &mut self.state);
    }

    fn on_close_event(&mut self) -> bool {
        use dynotest_app::windows::WSIdx::{ConfirmQuit, ConfirmUnsaved};
        if !self.control.is_buffer_saved() && !self.state.quit() {
            self.state.set_quitable(true);
            self.window_stack.set_open(ConfirmUnsaved, true);
            return false;
        }
        if self.state.quitable() {
            true
        } else {
            self.window_stack.set_open(ConfirmQuit, true);
            false
        }
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // every 15 minute save ( 900 sec == 15 min )
        std::time::Duration::from_secs(900)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.control.on_save_config();
        eframe::set_value(storage, APP_KEY, self);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.deinit();
    }
}

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Unable to create tokio's Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        })
    });

    let paths = DynoPaths::new();
    let log_dir = paths.get_cache_dir_file("logs/log.log");
    ignore_err!(init_logger(log_dir));
    Applications::run(paths)
}
