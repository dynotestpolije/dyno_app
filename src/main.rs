// #![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use dyno_core::{ignore_err, log, serde, tokio};
use dynotest_app::{
    control::DynoControl, init_logger, msg_dialog_err, state::DynoState, windows, PanelId, APP_KEY,
    PACKAGE_INFO, TOAST_MSG,
};
use eframe::egui::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
#[serde(default)]
pub struct Applications {
    control: DynoControl,

    #[serde(skip)]
    window_states: windows::WindowStack,

    state: DynoState,

    #[cfg(debug_assertions)]
    #[cfg_attr(debug_assertions, serde(skip))]
    debug: windows::DebugAction,
}

impl Default for Applications {
    fn default() -> Self {
        Self {
            window_states: windows::window_states_new(),
            control: DynoControl::new(),
            state: DynoState::new(),

            #[cfg(debug_assertions)]
            debug: Default::default(),
        }
    }
}

impl Applications {
    #[allow(clippy::new_ret_no_self)]
    pub fn run(control: DynoControl) {
        let log_dir = control.paths.get_cache_dir_file("logs/log.log");
        ignore_err!(init_logger(log_dir));
        let opt = control.app_config.app_options.main_window_opt();
        let app_creator: eframe::AppCreator = Box::new(|cc| {
            Box::new(
                cc.storage
                    .and_then(|s| eframe::get_value::<Self>(s, APP_KEY))
                    .unwrap_or_else(|| Self {
                        window_states: windows::window_states_new(),
                        control,
                        ..Default::default()
                    }),
            )
        });
        if let Err(err) = eframe::run_native(PACKAGE_INFO.app_name, opt, app_creator) {
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
                self.state.menubar(uibar);
                uibar.with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
                    widgets::global_dark_light_mode_switch(rtl_ui);
                    match self.control.api() {
                        Some(api) if api.is_logined() => {
                            if rtl_ui.button("Save to Server").clicked() {
                                log::info!("Saving to server...");
                                self.state.swap_show_save_server();
                            }
                            if rtl_ui.button("Logout").clicked() {
                                log::info!("Logout button clicked");
                                api.logout(self.control.tx().clone());
                            }
                        }
                        _ => {
                            if rtl_ui
                                .button("Login")
                                .on_hover_text(
                                    "login first to access server, like saving data to server.",
                                )
                                .clicked()
                            {
                                log::info!("Login bottom clicked");
                                self.state.swap_show_auth_window();
                            }
                        }
                    }
                });
            })
        });

        TopBottomPanel::bottom(PanelId::Bottom)
            .max_height(height * 0.05)
            .show_animated(ctx, self.state.show_bottom_panel(), |ui| {
                self.control.bottom_status(ui)
            });

        SidePanel::left(PanelId::Left)
            .min_width(width * 0.45)
            .max_width(width * 0.5)
            .show_animated(ctx, self.state.show_left_panel(), |ui| {
                self.control.left_panel(ui)
            });
        CentralPanel::default().show(ctx, |ui| self.control.right_panel(ui));
    }
}

impl eframe::App for Applications {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        {
            self.debug.show_window(ctx, self.control.buffer_mut());
        }
        for window in &mut self.window_states {
            window.show_window(ctx, &mut self.control, &mut self.state)
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
        self.control.on_pos_render(&mut self.state);
    }

    fn on_close_event(&mut self) -> bool {
        if !self.control.is_buffer_saved() && !self.state.quit() {
            self.state.set_show_buffer_unsaved(true);
            return false;
        }
        if self.state.quitable() {
            true
        } else {
            self.state.set_show_quitable(true);
            false
        }
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // every 15 minute save ( 900 sec == 15 min )
        std::time::Duration::from_secs(900)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
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

    let control = DynoControl::new();
    Applications::run(control)
}
