use crate::{config::ApplicationConfig, paths::DynoPaths, widgets::DynoWidgets};
use dyno_core::{serde, Cylinder, DynoConfig, MotorType, Stroke as InfoMotorStroke, Transmition};
use eframe::egui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
enum PanelSetting {
    #[default]
    Generic,
    Info,
    Style,
}

#[derive(Clone, Default)]
pub struct SettingWindow {
    panel: PanelSetting,
    edit_path: bool,
}

impl SettingWindow {
    pub fn new() -> Self {
        Self {
            panel: PanelSetting::default(),
            edit_path: false,
        }
    }

    #[allow(unused)]
    fn setting_generic(
        &mut self,
        ui: &mut Ui,
        app_config: &mut ApplicationConfig,
        paths: &mut DynoPaths,
    ) {
        let path_ui = |ui: &mut Ui| {};
        CollapsingHeader::new("✒ Paths")
            .default_open(true)
            .show(ui, |path_ui| paths.draw(path_ui, &mut self.edit_path));
        ui.separator();

        CollapsingHeader::new(" Configurations")
            .default_open(true)
            .show(ui, |config_ui| {
                app_config.draw(config_ui);
            });
    }

    #[allow(unused)]
    fn setting_info(&mut self, ui: &mut Ui, conf: &mut DynoConfig) {
        match &mut conf.motor_type {
            MotorType::Electric(_) => todo!(),
            MotorType::Engine(ref mut info) => {
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
                    horzui.selectable_value_from_iter(&mut info.cylinder, Cylinder::into_iter());
                    horzui.separator();
                    horzui
                        .selectable_value_from_iter(&mut info.stroke, InfoMotorStroke::into_iter());
                    horzui.separator();
                    horzui.selectable_value_from_iter(
                        &mut info.transmition,
                        Transmition::into_iter(),
                    );
                });
            }
        }
    }
}

impl super::WindowState for SettingWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
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
                    .show(ui, |scr_ui| {
                        let crate::control::DynoControl {
                            paths,
                            app_config,
                            config,
                            ..
                        } = control;
                        match self.panel {
                            PanelSetting::Generic => {
                                self.setting_generic(scr_ui, app_config, paths)
                            }
                            PanelSetting::Info => self.setting_info(scr_ui, config),
                            PanelSetting::Style => {
                                ctx.settings_ui(scr_ui);
                                scr_ui.separator();
                                ctx.inspection_ui(scr_ui);
                            }
                        };
                    });
            });
    }
}
