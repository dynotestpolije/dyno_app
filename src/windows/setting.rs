use crate::{config::ApplicationConfig, paths::DynoPaths, row_label_value, widgets::DynoWidgets};
use dyno_core::{
    serde, Cylinder, DynoConfig, ElectricMotor, InfoMotor, MotorType, Stroke as InfoMotorStroke,
    Transmition,
};
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
    pub fn setting_info(ui: &mut Ui, conf: &mut DynoConfig) {
        let info_motor_ui = |im_ui: &mut Ui| match &mut conf.motor_type {
            MotorType::Electric(ElectricMotor { name }) => {
                im_ui.add(TextEdit::singleline(name).hint_text("isi nama motor"));
            }
            MotorType::Engine(InfoMotor {
                name,
                cc,
                cylinder,
                stroke,
                transmition,
            }) => {
                row_label_value!(
                    im_ui,
                    TextEdit::singleline(name).hint_text("isi nama motor"),
                    "Motor Name",
                    "nama motor (hanya untuk informasi data)"
                );
                im_ui.end_row();
                row_label_value!(
                    im_ui,
                    Slider::new(cc, 20u32..=2000u32).suffix(" cc"),
                    "Motor Name",
                    "nama motor (hanya untuk informasi data)"
                );
                im_ui.end_row();
                row_label_value!(
                    im_ui => im_ui.combobox_from_iter(
                        "Cylinder of Engine",
                        cylinder,
                        Cylinder::into_iter()
                    ),
                    "Cylinder",
                    "silinder mesin (hanya untuk informasi data dan perhitungan rpm engine)"
                );
                im_ui.end_row();
                row_label_value!(
                    im_ui => im_ui.combobox_from_iter(
                        "Stroke of Engine",
                        stroke,
                        InfoMotorStroke::into_iter()
                    ),
                    "Stroke",
                    "stroke mesin (hanya untuk informasi data dan perhitungan rpm engine)"
                );
                im_ui.end_row();
                row_label_value!(
                    im_ui => im_ui.combobox_from_iter(
                        "Transmitions of Engine",
                        transmition,
                        Transmition::into_iter()
                    ),
                    "Transmitions",
                    "transmisi mesin (hanya untuk informasi data)"
                );
            }
        };
        let other_motor_info_config_ui = |ui: &mut Ui| {
            // diameter_roller: length::Metres,
            row_label_value!(
                ui, DragValue::new(conf.diameter_roller.value_mut()).suffix(" m"),
                "Diameter Roller",
                "Diameter dari roller dynotest chasis (digunakan untuk menghitung informasi data sensor)"
            );
            ui.end_row();
            // diameter_roller_beban: length::Metres,
            row_label_value!(
                ui, DragValue::new(conf.diameter_roller_beban.value_mut()).suffix(" m"),
                "Diameter Roller Beban",
                "Beban dari roller dynotest chasis (digunakan untuk menghitung informasi data sensor)"
            );
            ui.end_row();
            // diameter_gear_encoder: length::Metres,
            row_label_value!(
                ui, DragValue::new(conf.diameter_gear_encoder.value_mut()).suffix(" m"),
                "Diameter Gear Encoder",
                "Diameter dari gear yang terdapat pada Encoder dynotest chasis (digunakan untuk menghitung informasi data sensor)"
            );
            ui.end_row();
            // diameter_gear_beban: length::Metres,
            row_label_value!(
                ui, DragValue::new(conf.diameter_gear_beban.value_mut()).suffix(" m"),
                "Diameter Gear Beban",
                "Diameter dari gear yang terdapat pada roller Beban dynotest chasis (digunakan untuk menghitung informasi data sensor)"
            );
            ui.end_row();
            // jarak_gear: length::Metres,
            row_label_value!(
                ui, DragValue::new(conf.jarak_gear.value_mut()).suffix(" m"),
                "Jarak Antar Gear",
                "jarak diantara gear pada roller Beban dan sensor Encoder dynotest chasis (digunakan untuk menghitung informasi data sensor)"
            );
            ui.end_row();
            // berat_beban: weight::KiloGram,
            row_label_value!(
                ui, DragValue::new(conf.berat_beban.value_mut()).suffix(" kg"),
                "Berat Roller Beban",
                "berat roller Beban pada dynotest chasis (digunakan untuk menghitung informasi data sensor)"
            );
        };
        CollapsingHeader::new("Info Motor Config")
            .id_source("dyno_info_motor_config_id")
            .show(ui, |ui| {
                Grid::new("dyno_info_motor_config_grid_id")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, info_motor_ui)
            });
        CollapsingHeader::new("Data Configuration")
            .id_source("dyno_configuration_id")
            .show(ui, |ui| {
                Grid::new("dyno_configuration_grid_id")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, other_motor_info_config_ui)
            });
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
                            PanelSetting::Info => {
                                scr_ui.heading("Info Setting");
                                scr_ui.separator();
                                Self::setting_info(scr_ui, config)
                            }
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
