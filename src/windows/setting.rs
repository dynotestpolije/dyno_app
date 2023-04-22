use eframe::egui::*;
use std::sync::{Arc, RwLock};

use crate::widgets::DynoWidgets;
use crate::{config::DynoConfig, paths::DynoPaths};
use dyno_types::infomotor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
enum PanelSetting {
    #[default]
    Generic,
    Info,
    Style,
}

#[derive(Clone, Default)]
pub struct SettingWindow {
    paths: Arc<RwLock<DynoPaths>>,
    config: Arc<RwLock<DynoConfig>>,
    info: Arc<RwLock<infomotor::InfoMotor>>,

    panel: PanelSetting,
    edit_path: bool,
}

impl SettingWindow {
    pub fn new(
        paths: Arc<RwLock<DynoPaths>>,
        config: Arc<RwLock<DynoConfig>>,
        info: Arc<RwLock<infomotor::InfoMotor>>,
    ) -> Self {
        Self {
            paths,
            config,
            info,
            panel: PanelSetting::default(),
            edit_path: false,
        }
    }

    #[allow(unused)]
    fn setting_generic(&mut self, ui: &mut Ui) {
        let path_ui = |ui: &mut Ui| {
            if let Ok(mut paths) = self.paths.write() {
                paths.draw(ui, &mut self.edit_path)
            }
        };
        CollapsingHeader::new("✒ Paths")
            .default_open(true)
            .show(ui, path_ui);
        ui.separator();

        let config_ui = |ui: &mut Ui| {
            if let Ok(mut config) = self.config.write() {
                config.draw(ui);
            }
        };
        CollapsingHeader::new(" Configurations")
            .default_open(true)
            .show(ui, config_ui);
    }

    #[allow(unused)]
    fn setting_info(&mut self, ui: &mut Ui) {
        let Ok(mut info) = self.info.write() else {
            return;
        };
        let resp = ui.heading("Info Setting");
        ui.separator();
        ui.horizontal_wrapped(|horzui| {
            horzui.add(TextEdit::singleline(&mut info.name).hint_text("isi nama motor"));
            horzui.separator();
            horzui.add(
                DragValue::new(&mut info.cc)
                    .speed(1)
                    .prefix("Volume Cilinder: ")
                    .suffix(" cc")
                    .min_decimals(10)
                    .max_decimals(30),
            );
        });
        ui.separator();
        ui.horizontal_wrapped(|horzui| {
            horzui.selectable_value_from_iter(&mut info.cylinder, infomotor::Cylinder::into_iter());
            horzui.separator();
            horzui.selectable_value_from_iter(&mut info.stroke, infomotor::Stroke::into_iter());
            horzui.separator();
            horzui.selectable_value_from_iter(
                &mut info.transmition,
                infomotor::Transmition::into_iter(),
            );
        });
        ui.separator();
        ui.add(
            DragValue::new(&mut info.tire_diameter)
                .speed(1)
                .prefix("Diameter Ban: ")
                .suffix(" inch")
                .min_decimals(10)
                .max_decimals(50),
        );
    }
}

impl super::WindowState for SettingWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
        state: &mut crate::state::DynoState,
    ) {
        Window::new("Dyno Control Settings")
            .id(Id::new("id_control_setting"))
            .open(state.show_config_mut())
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.horizontal(|ui| {
                        use PanelSetting::*;
                        ui.selectable_value(&mut self.panel, Generic, stringify!(Generic));
                        ui.selectable_value(&mut self.panel, Info, stringify!(InfoMotor));
                        ui.selectable_value(&mut self.panel, Style, stringify!(Style));
                    });
                });
                ui.separator();

                ScrollArea::vertical()
                    .id_source("dyno_settings")
                    .show(ui, |scr_ui| match self.panel {
                        PanelSetting::Generic => self.setting_generic(scr_ui),
                        PanelSetting::Info => self.setting_info(scr_ui),
                        PanelSetting::Style => {
                            ctx.settings_ui(scr_ui);
                            scr_ui.separator();
                            ctx.inspection_ui(scr_ui);
                        }
                    });
            });
    }
}
