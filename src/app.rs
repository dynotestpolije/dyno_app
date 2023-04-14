use crate::control::DynoControl;
use dyno_types::{data_buffer::Data, log, Numeric};
use dynotest_app::{
    config::CoreConfig,
    paths::DynoPaths,
    widgets::{button::ButtonKind, gauges::Gauges, logger, segment_display, toast::Toasts},
    window, APP_KEY, PACKAGE_INFO,
};

use eframe::egui::*;

use crate::condition::{Condition, FileType, OperatorData};

#[allow(dead_code)]
pub enum PanelId {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

impl From<PanelId> for Id {
    fn from(value: PanelId) -> Self {
        match value {
            PanelId::Top => Id::new("E3221406_dynotest_top_panel"),
            PanelId::Bottom => Id::new("E3221406_dynotest_bottom_panel"),
            PanelId::Left => Id::new("E3221406_dynotest_left_panel"),
            PanelId::Right => Id::new("E3221406_dynotest_right_panel"),
            PanelId::Center => Id::new("E3221406_central_panel"),
        }
    }
}

#[cfg(debug_assertions)]
mod dyno_debug {
    use dyno_types::chrono::NaiveDateTime;
    use dynotest_app::widgets::DynoWidgets;

    #[derive(Debug, Default)]
    pub struct DebugAction {
        rpm: f64,
        speed: f64,
        torque: f64,
        hp: f64,
        odo: f64,
        display_style: dynotest_app::widgets::DisplayStylePreset,
    }

    impl DebugAction {
        pub fn get_preset(&self) -> dynotest_app::widgets::DisplayStylePreset {
            self.display_style
        }
        pub fn draw(
            &mut self,
            ctx: &super::Context,
            buffer: &mut dyno_types::data_buffer::BufferData,
        ) {
            use dyno_types::convertions::prelude::*;
            super::Window::new("Debug Window").show(ctx, |ui| {
                let Self {
                    rpm,
                    speed,
                    torque,
                    hp,
                    odo,
                    display_style,
                } = self;
                ui.label("RPM");
                ui.add(super::DragValue::new(rpm).clamp_range(0.0..=15_000.0));
                ui.separator();
                ui.label("SPEED");
                ui.add(super::DragValue::new(speed).clamp_range(0.0..=440.0));
                ui.separator();
                ui.label("TORQUE");
                ui.add(super::DragValue::new(torque).clamp_range(0.0..=200.0));
                ui.separator();
                ui.label("HP");
                ui.add(super::DragValue::new(hp).clamp_range(0.0..=200.0));
                ui.separator();
                ui.label("ODO");
                ui.add(super::DragValue::new(odo).clamp_range(0.0..=1.0));

                ui.separator();
                ui.selectable_value_from_iter(display_style, display_style.get_iter())
            });

            let data = super::Data {
                speed: KilometresPerHour::new(self.speed),
                rpm: RotationPerMinute::new(self.rpm),
                odo: KiloMetres::new(self.odo),
                horsepower: self.hp,
                torque: self.torque,
                temp: Celcius::new(0.0),
                time_stamp: NaiveDateTime::MIN,
            };
            buffer.push_data(data);
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Applications {
    name: String,
    paths: DynoPaths,
    control: DynoControl,
    config: CoreConfig,

    #[serde(skip)]
    #[serde(default)]
    conds: Condition,

    #[serde(skip)]
    #[serde(default)]
    toast: Toasts,

    #[cfg_attr(debug_assertions, serde(skip), serde(default))]
    debug: dyno_debug::DebugAction,
}

impl Default for Applications {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            config: Default::default(),
            name: PACKAGE_INFO.app_name.to_string(),
            control: DynoControl::new(),
            toast: Default::default(),
            conds: Condition::new(),
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

    #[inline]
    fn reload(&mut self) {
        todo!("reload machanism");
        // *self = Self::default();
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
                            format!("{odo:.1}", odo = data.odo.to_float()),
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
                            format!("{speed:.1}", speed = data.speed.to_float()),
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
                self.conds.menubar(uibar)
            })
        });
        TopBottomPanel::bottom(PanelId::Bottom).show_animated(
            ctx,
            self.conds.get_show_bottom_panel(),
            |uibottom| {
                uibottom.horizontal(|vertui| {
                    self.control.show_status(vertui);
                });
            },
        );
        SidePanel::left(PanelId::Left)
            .min_width(width * 0.3)
            .max_width(width * 0.5)
            .show_animated(ctx, self.conds.get_show_left_panel(), uigroup_column_left);
        CentralPanel::default().show(ctx, uigroup_column_right);
    }
    #[inline]
    fn windows_draw(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if cfg!(debug_assertions) {
            self.debug.draw(ctx, self.control.buffer_mut());
        }
        if self.conds.get_show_logger_window() {
            logger::logger_ui(ctx);
        }
        if self.conds.get_show_about() {
            window::show_about(ctx, self.conds.get_show_about_mut());
        }
        if self.conds.get_show_help() {
            window::show_help(ctx, self.conds.get_show_help_mut());
        }
        if self.conds.get_show_config() {
            self.control.show_setting(
                ctx,
                self.conds.get_show_config_mut(),
                &mut self.paths,
                &mut self.config,
            )
        }

        if self.conds.get_confirm_quit() {
            match window::confirm_quit(ctx, self.conds.get_confirm_quit_mut()) {
                window::ConfirmOption::Yes => {
                    self.conds.set_allow_close(true);
                    self.conds.set_confirm_quit(false);
                    frame.close();
                }
                window::ConfirmOption::No => {
                    self.conds.set_allow_close(false);
                    self.conds.set_confirm_quit(false);
                }
                window::ConfirmOption::Def => {}
            }
        }
        self.toast.show(ctx);
    }
    #[inline]
    fn on_conditions(&mut self) {
        macro_rules! save_buffer_macros {
            ($tp: expr) => {{
                let tp = $tp;
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
            }};
        }

        match (
            self.conds.get_confirm_quit(),
            self.conds.get_confirm_reload(),
            self.conds.get_buffer_saved(),
        ) {
            (_, true, true) => self.reload(), // want to reload and the buffer is saved
            (q, r, false) if q || r => match DynoControl::popup_unsaved() {
                ButtonKind::Ok => {
                    self.conds.set_buffer_saved(true);
                    self.conds.set_operator(OperatorData::Noop);
                    save_buffer_macros!(FileType::All);
                }
                ButtonKind::Cancel => {
                    self.conds.set_confirm_reload(false);
                    self.conds.set_confirm_quit(false);
                }
                _ => self.conds.set_buffer_saved(true),
            },
            _ => (),
        }

        match self.conds.get_operator() {
            OperatorData::Noop => {}
            OperatorData::SaveFile(tp) => {
                log::info!("on Saving a File of {}", tp.as_str());
                self.conds.set_buffer_saved(true);
                self.conds.set_operator(OperatorData::Noop);
                save_buffer_macros!(tp);
            }
            OperatorData::OpenFile(tp) => {
                log::info!("on Saving a File of {}", tp.as_str());
                if !self.conds.get_buffer_saved() {
                    match DynoControl::popup_unsaved() {
                        ButtonKind::Ok => {
                            self.conds.set_buffer_saved(true);
                            save_buffer_macros!(tp);
                        }
                        ButtonKind::Cancel => self.conds.set_operator(OperatorData::Noop),
                        _ => self.conds.set_buffer_saved(true),
                    }
                } else {
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
                    self.conds.set_operator(OperatorData::Noop);
                }
            }
        }
    }
}

impl eframe::App for Applications {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let last_data = self.control.last_buffer();
        self.windows_draw(ctx, frame);
        self.main_panels_draw(ctx, &last_data);
        self.on_conditions();
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        self.control.on_pos_render();
    }

    fn on_close_event(&mut self) -> bool {
        if self.conds.get_allow_close() {
            return true;
        }
        self.conds.set_confirm_quit(true);
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
