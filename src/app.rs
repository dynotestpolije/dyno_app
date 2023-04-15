use crate::{
    control::DynoControl,
    state::{AppState, FileType, OperatorData},
};
use dyno_types::{data_buffer::Data, Numeric};
use dynotest_app::{
    config::CoreConfig,
    paths::DynoPaths,
    widgets::{button::ButtonKind, gauges::Gauges, segment_display, toast::Toasts},
    window, PanelId, APP_KEY, PACKAGE_INFO,
};
use eframe::egui::*;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Applications {
    name: String,
    paths: DynoPaths,
    control: DynoControl,
    config: CoreConfig,

    #[serde(skip)]
    #[serde(default)]
    state: AppState,

    #[serde(skip)]
    #[serde(default)]
    toast: Toasts,

    #[cfg_attr(debug_assertions, serde(skip), serde(default))]
    debug: crate::debug::DebugAction,
}

impl Default for Applications {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            config: Default::default(),
            name: PACKAGE_INFO.app_name.to_string(),
            control: DynoControl::new(),
            toast: Default::default(),
            state: AppState::new(),
            debug: Default::default(),
        }
    }
}

impl Applications {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        paths: DynoPaths,
        config: CoreConfig,
    ) -> Box<dyn eframe::App + 'static> {
        let mut slf = cc
            .storage
            .and_then(|s| eframe::get_value::<Self>(s, APP_KEY))
            .unwrap_or_default();
        slf.config.check_is_changed(&config);
        slf.paths.check_is_changed(&paths);
        Box::new(slf)
    }
}

impl Applications {
    #[inline]
    fn main_panels_draw(&mut self, ctx: &Context, data: &Data) {
        let uigroup_column_left = |left_ui: &mut Ui| {
            let width = left_ui.available_width() / 2.1;
            left_ui.horizontal(|uihorz| {
                uihorz.add(Gauges::speed(data.speed).diameter(width));
                uihorz.add(Gauges::rpm(data.rpm).diameter(width));
            });
            left_ui.separator();
            left_ui.horizontal(|uihorz| {
                uihorz.add(Gauges::horsepower(data.horsepower).diameter(width));
                uihorz.add(Gauges::torque(data.torque).diameter(width));
            });
        };
        let uigroup_column_right = |right_ui: &mut Ui| {
            right_ui.columns(2, |segments_ui| {
                segments_ui[0].group(|uigroup_inner| {
                    let digit_height = uigroup_inner.available_width() * 0.15;
                    uigroup_inner.heading("ODO (Km)");
                    uigroup_inner.add({
                        let mut segment = segment_display::SegmentedDisplay::dyno_seven_segment(
                            format!("{:07.2}", data.odo.to_float()),
                        )
                        .digit_height(digit_height);
                        if cfg!(debug_assertions) {
                            segment = segment.style_preset(self.debug.get_preset());
                        }
                        segment
                    });
                });
                segments_ui[1].group(|uigroup_inner| {
                    let digit_height = uigroup_inner.available_width() * 0.15;
                    uigroup_inner.heading("SPEED (Km/H)");
                    uigroup_inner.add({
                        let mut segment = segment_display::SegmentedDisplay::dyno_seven_segment(
                            format!("{:06.2}", data.speed.to_float()),
                        )
                        .digit_height(digit_height);
                        if cfg!(debug_assertions) {
                            segment = segment.style_preset(self.debug.get_preset());
                        }
                        segment
                    });
                });
            });
            right_ui.separator();
            self.control.show_plot(right_ui);
        };
        let width = ctx.available_rect().width();
        TopBottomPanel::top(PanelId::Top).show(ctx, |ui| {
            menu::bar(ui, |uibar| {
                widgets::global_dark_light_mode_switch(uibar);
                uibar.separator();
                self.state.menubar(uibar)
            })
        });
        TopBottomPanel::bottom(PanelId::Bottom).show_animated(
            ctx,
            self.state.show_bottom_panel(),
            |uibottom| {
                uibottom.horizontal(|vertui| {
                    self.control.show_status(vertui);
                });
            },
        );
        SidePanel::left(PanelId::Left)
            .min_width(width * 0.3)
            .max_width(width * 0.5)
            .show_animated(ctx, self.state.show_left_panel(), uigroup_column_left);
        CentralPanel::default().show(ctx, uigroup_column_right);
    }
    #[inline]
    fn windows_draw(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if cfg!(debug_assertions) {
            self.debug.draw(ctx, self.control.buffer_mut());
        }
        window::show_logger(ctx, self.state.show_logger_window_mut());
        window::show_about(ctx, self.state.show_about_mut());
        window::show_help(ctx, self.state.show_help_mut());
        self.control.show_setting(
            ctx,
            self.state.show_config_mut(),
            &mut self.paths,
            &mut self.config,
        );

        if self.state.confirm_quit() {
            match DynoControl::popup_unsaved(ctx, self.state.buffer_saved()) {
                ButtonKind::No => match window::confirm_quit(ctx) {
                    ButtonKind::Ok => {
                        self.state.set_confirm_quit(false);
                        self.state.set_allow_close(true);
                        frame.close();
                    }
                    ButtonKind::No => {
                        self.state.set_confirm_quit(false);
                        self.state.set_allow_close(false);
                    }
                    _ => {}
                },
                ButtonKind::Cancel => {
                    self.state.set_confirm_quit(false);
                    self.state.set_confirm_reload(false);
                }
                ButtonKind::Save => self
                    .state
                    .set_operator(OperatorData::SaveFile(FileType::All)),
                _ => {}
            }
        }
        self.toast.show(ctx);
    }
    #[inline]
    fn on_conditions(&mut self, ctx: &Context) {
        let mut save_buffer = |tp: FileType| {
            let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
            match self.control.on_save(tp, &dirpath) {
                Ok(Some(path)) => {
                    self.toast
                        .info(format!("Saved {} file: {}", tp.as_str(), path.display()));
                }
                Err(err) => {
                    self.toast
                        .error(format!("Error on Saving {}: {err}", tp.as_str()));
                }
                _ => {}
            };
        };

        match self.state.operator() {
            OperatorData::Noop => {}
            OperatorData::SaveFile(tp) => {
                self.state.set_buffer_saved(true);
                self.state.set_operator(OperatorData::Noop);
                save_buffer(tp);
            }
            OperatorData::OpenFile(tp) => {
                match DynoControl::popup_unsaved(ctx, self.state.buffer_saved()) {
                    ButtonKind::No => {
                        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
                        match self.control.on_open(tp, dirpath) {
                            Ok(p) => {
                                if let Some(path) = p {
                                    self.toast.info(format!(
                                        "Opened {} file: {}",
                                        tp.as_str(),
                                        path.display()
                                    ));
                                }
                            }
                            Err(e) => {
                                self.toast.error(format!("Failed Opening File: {e}"));
                            }
                        }
                        self.state.set_operator(OperatorData::Noop);
                    }
                    ButtonKind::Save => {
                        self.state.set_buffer_saved(true);
                        save_buffer(tp);
                    }
                    ButtonKind::Cancel => self.state.set_operator(OperatorData::Noop),
                    _ => {}
                }
            }
        }
    }
}

impl eframe::App for Applications {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let last_data = self.control.last_buffer();
        self.state.set_buffer_saved(self.control.buffer_empty());

        self.windows_draw(ctx, frame);
        self.main_panels_draw(ctx, &last_data);
        self.on_conditions(ctx);
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        self.control.on_pos_render();
    }

    fn on_close_event(&mut self) -> bool {
        if self.state.allow_close() {
            return true;
        }
        self.state.set_confirm_quit(true);
        false
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // every 15 minute save ( 900 sec == 15 min )
        std::time::Duration::from_secs(900)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }
}
