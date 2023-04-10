use crate::controller::Controller;
use dyno_types::{data_buffer::Data, Numeric};
use dynotest_app::{
    widgets::{button::ButtonKind, gauges::Gauges, segment_display, toast::Toasts},
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

impl Into<Id> for PanelId {
    #[inline(always)]
    fn into(self) -> Id {
        match self {
            Self::Top => Id::new("dyno_top_panel"),
            Self::Bottom => Id::new("dyno_bottom_panel"),
            Self::Left => Id::new("dyno_left_panel"),
            Self::Right => Id::new("dyno_right_panel"),
            Self::Center => Id::new("central_panel"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Applications {
    name: String,
    controller: Controller,
    conds: Condition,

    #[serde(skip)]
    #[serde(default)]
    toast: Toasts,
}

impl Default for Applications {
    fn default() -> Self {
        Self {
            name: PACKAGE_INFO.app_name.to_string(),
            controller: Default::default(),
            toast: Default::default(),
            conds: Condition::new(),
        }
    }
}

impl Applications {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        controller: Controller,
    ) -> Box<dyn eframe::App + 'static> {
        let mut slf = cc
            .storage
            .and_then(|s| eframe::get_value::<Self>(s, APP_KEY))
            .unwrap_or_default();

        slf.controller.check_if_change(&controller);
        Box::new(slf)
    }

    #[inline]
    fn reload(&mut self) {
        let mut slf = Self::default();
        slf.controller.check_if_change(&self.controller);
        *self = slf;
    }

    #[inline]
    fn left_panel(&mut self, ui: &mut Ui, data: &Data) {
        Grid::new("grid_gauges").num_columns(2).show(ui, |gridui| {
            let Data {
                speed,
                torque,
                horsepower,
                rpm,
                ..
            } = data;
            let width = gridui.available_width();
            gridui.add(Gauges::speed(*speed).diameter(width));
            gridui.add(Gauges::rpm(*rpm).diameter(width));
            gridui.end_row();
            gridui.add(Gauges::horsepower(*horsepower).diameter(width));
            gridui.add(Gauges::torque(*torque).diameter(width));
            gridui.end_row();
        });
    }

    #[inline]
    fn main_panel(&mut self, ui: &mut Ui, data: &Data) {
        ui.horizontal(|horzui| {
            let Data { speed, odo, .. } = data;
            horzui.group(|uigrup| {
                uigrup.heading("ODO (Km)");
                uigrup.add(segment_display::SegmentedDisplay::dyno_seven_segment(
                    format!("{odo:.2}", odo = odo.to_float()),
                ));
            });
            horzui.separator();
            horzui.group(|uigrup| {
                uigrup.heading("SPEED (Km/H)");
                uigrup.add(segment_display::SegmentedDisplay::dyno_seven_segment(
                    format!("{speed:.2}", speed = speed.to_float()),
                ));
            });
        });
        ui.separator();
        self.controller.show_plot(ui);
    }
}

impl eframe::App for Applications {
    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        self.controller.on_pos_render();
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(3));

        if cfg!(debug_assertions) {
            if let Some(pos) = ctx.pointer_latest_pos() {
                let info = self.controller.info_motor().clone();
                let buf = self.controller.buffer_mut();
                let serdata = dyno_types::SerialData {
                    time: 100,
                    pulse_encoder: (pos.y + pos.x) as u32 * 10,
                    pulse_rpm: pos.y as u32 * 100,
                    temperature: pos.y / 10.,
                };
                buf.push_data(Data::from_serial(&info, serdata));
            }
        }

        let last_data = self.controller.last_buffer();
        TopBottomPanel::top(PanelId::Top).show(ctx, |ui| {
            eframe::egui::trace!(ui, "dyno_top_panel");
            menu::bar(ui, |uibar| {
                widgets::global_dark_light_mode_switch(uibar);
                uibar.separator();
                self.conds.menubar(uibar)
            })
        });

        SidePanel::left(PanelId::Left)
            .min_width(200.0)
            .max_width(ctx.used_size().x * 0.5)
            .resizable(true)
            .show(ctx, |ui| {
                eframe::egui::trace!(ui, "dyno_left_panel");
                ui.vertical_centered(|ui| self.left_panel(ui, &last_data))
            });

        TopBottomPanel::bottom(PanelId::Bottom).show(ctx, |ui| {
            eframe::egui::trace!(ui, "dyno_bottom_panel");
            ui.horizontal(|ui| {
                self.controller.show_status(ui);
            })
        });

        CentralPanel::default().show(ctx, |ui| {
            eframe::egui::trace!(ui, "dyno_main_panel");
            ui.vertical_centered_justified(|ui| self.main_panel(ui, &last_data))
        });

        if self.conds.get_show_about() {
            window::show_about(ctx, self.conds.get_show_about_mut());
        }
        if self.conds.get_show_help() {
            window::show_help(ctx, self.conds.get_show_help_mut());
        }
        if self.conds.get_show_config() {
            self.controller
                .show_setting(ctx, self.conds.get_show_config_mut())
        }

        if self.conds.get_confirm_quit() {
            if window::confirm_quit(ctx, self.conds.get_confirm_quit_mut()) {
                self.conds.set_allow_close(true);
                frame.close();
            }
        }
        self.toast.show(ctx);
        self.on_conditions();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    fn on_close_event(&mut self) -> bool {
        self.conds.set_confirm_quit(true);
        dbg!(self.conds.get_allow_close())
    }
    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(120)
    }
}

impl Applications {
    #[inline]
    fn on_conditions(&mut self) {
        macro_rules! save_buffer_macros {
            ($tp: expr) => {{
                let tp = $tp;
                match self.controller.on_save(tp) {
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

        if self.conds.get_confirm_reload() {
            if !self.conds.get_buffer_saved() {
                match Controller::popup_unsaved() {
                    ButtonKind::Ok => {
                        self.conds.set_buffer_saved(true);
                        self.conds.set_operator(OperatorData::Noop);
                        save_buffer_macros!(FileType::All);
                    }
                    ButtonKind::Cancel => self.conds.set_confirm_reload(false),
                    _ => self.conds.set_buffer_saved(true),
                }
            }
            self.reload();
        }

        match self.conds.get_operator() {
            OperatorData::Noop => {}
            OperatorData::SaveFile(tp) => {
                self.conds.set_buffer_saved(true);
                self.conds.set_operator(OperatorData::Noop);
                save_buffer_macros!(tp);
            }
            OperatorData::OpenFile(tp) => {
                if !self.conds.get_buffer_saved() {
                    match Controller::popup_unsaved() {
                        ButtonKind::Ok => {
                            self.conds.set_buffer_saved(true);
                            save_buffer_macros!(tp);
                        }
                        ButtonKind::Cancel => self.conds.set_operator(OperatorData::Noop),
                        _ => self.conds.set_buffer_saved(true),
                    }
                } else {
                    match self.controller.on_open(tp) {
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
