use crate::{
    config::ApplicationConfig,
    paths::DynoPaths,
    row_label_value,
    widgets::{button::ButtonExt, DynoWidgets},
};
use dyno_core::{serde, Cylinder, DynoConfig, MotorType, Stroke as InfoMotorStroke, Transmition};
use eframe::egui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub enum PanelSetting {
    #[default]
    Generic,
    Info,
    Style,
}

#[derive(Clone, Default)]
pub struct SettingWindow {
    open: bool,
    panel: PanelSetting,
    edit_path: bool,
}

impl SettingWindow {
    pub fn new() -> Self {
        Self {
            panel: PanelSetting::default(),
            ..Default::default()
        }
    }
    #[inline]
    #[allow(unused)]
    pub fn open_on_panel_info(&mut self) {
        self.open = true;
        self.panel = PanelSetting::Info;
    }
    #[inline]
    #[allow(unused)]
    pub fn open_on_panel_generic(&mut self) {
        self.open = true;
        self.panel = PanelSetting::Generic;
    }
    #[inline]
    #[allow(unused)]
    pub fn open_on_panel_style(&mut self) {
        self.open = true;
        self.panel = PanelSetting::Style;
    }

    #[allow(unused)]
    fn setting_generic(
        ui: &mut Ui,
        app_config: &mut ApplicationConfig,
        paths: &mut DynoPaths,
        edit_path: &mut bool,
    ) {
        let path_ui = |ui: &mut Ui| {};
        CollapsingHeader::new("✒ Paths")
            .default_open(true)
            .show(ui, |path_ui| paths.draw(path_ui, edit_path));
        ui.separator();

        CollapsingHeader::new(" Configurations")
            .default_open(true)
            .show(ui, |config_ui| {
                app_config.draw(config_ui);
            });
    }

    #[allow(unused)]
    pub fn setting_info(ui: &mut Ui, conf: &mut DynoConfig) {
        CollapsingHeader::new("Info Motor Config")
            .id_source("dyno_info_motor_config_id")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut conf.motor_type,
                        MotorType::Engine,
                        stringify!(Engine),
                    );
                    ui.selectable_value(
                        &mut conf.motor_type,
                        MotorType::Electric,
                        stringify!(Electric),
                    );
                });
                let info_motor_ui = |im_ui: &mut Ui| match conf.motor_type {
                    MotorType::Electric => {
                        im_ui.add(
                            TextEdit::singleline(&mut conf.motor_info.name)
                                .hint_text("isi nama motor"),
                        );
                    }
                    MotorType::Engine => {
                        row_label_value!(
                            im_ui,
                            TextEdit::singleline(&mut conf.motor_info.name)
                                .hint_text("isi nama motor"),
                            "Motor Name",
                            "nama motor (hanya untuk informasi data)"
                        );
                        im_ui.end_row();
                        row_label_value!(
                            im_ui,
                            Slider::new(&mut conf.motor_info.cc, 20u32..=2000u32).suffix(" cc"),
                            "CC",
                            "nama motor (hanya untuk informasi data)"
                        );
                        im_ui.end_row();
                        row_label_value!(
                            im_ui => im_ui.combobox_from_iter(
                                "Cylinder of Engine",
                                &mut conf.motor_info.cylinder,
                                Cylinder::into_iter()
                            ),
                            "Cylinder",
                            "silinder mesin (hanya untuk informasi data dan perhitungan rpm engine)"
                        );
                        im_ui.end_row();
                        row_label_value!(
                            im_ui => im_ui.combobox_from_iter(
                                "Stroke of Engine",
                                &mut conf.motor_info.stroke,
                                InfoMotorStroke::into_iter()
                            ),
                            "Stroke",
                            "stroke mesin (hanya untuk informasi data dan perhitungan rpm engine)"
                        );
                        im_ui.end_row();
                        row_label_value!(
                            im_ui => im_ui.combobox_from_iter(
                                "Transmitions of Engine",
                                &mut conf.motor_info.transmition,
                                Transmition::into_iter()
                            ),
                            "Transmitions",
                            "transmisi mesin (hanya untuk informasi data)"
                        );
                    }
                };
                Grid::new("dyno_info_motor_config_grid_id")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, info_motor_ui)
            });

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
        CollapsingHeader::new("Data Configuration")
            .id_source("dyno_configuration_id")
            .default_open(true)
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
        _state: &mut crate::state::DynoState,
    ) {
        let mut open = self.open;
        let mut edit_path = self.edit_path;
        let mut panel = self.panel;

        Window::new("Dyno Control Settings")
            .id(Id::new("id_control_setting"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.horizontal(|ui| {
                        use PanelSetting::*;
                        ui.selectable_value(&mut panel, Generic, stringify!(Generic));
                        ui.selectable_value(&mut panel, Info, stringify!(InfoMotor));
                        ui.selectable_value(&mut panel, Style, stringify!(Style));
                    });
                });
                ui.separator();
                let scroll_area_ui = |scr_ui: &mut Ui| {
                    scr_ui.add_space(20.);
                    match panel {
                        PanelSetting::Generic => Self::setting_generic(
                            scr_ui,
                            &mut control.app_config,
                            &mut control.paths,
                            &mut edit_path,
                        ),
                        PanelSetting::Info => {
                            scr_ui.heading("Info Setting");
                            scr_ui.separator();
                            match control.service.serial() {
                                Some(serial) => {
                                    let serial_open = serial.is_open();
                                    let (status, color) = if serial_open {
                                        ("STATUS: Running", Color32::BLUE)
                                    } else {
                                        ("STATUS: Connected", Color32::GREEN)
                                    };
                                    let info = serial.get_info();
                                    Label::new(RichText::new(status).color(color))
                                        .ui(scr_ui)
                                        .on_hover_text(format!(
                                            "PORT INFO: [{}] ({}:{})",
                                            info.port_name, info.vid, info.pid
                                        ));
                                    scr_ui.separator();
                                    let btn_start = scr_ui
                                        .small_play_button()
                                        .on_hover_text("Click to Start the Service");
                                    let btn_reset = scr_ui
                                        .small_reset_button()
                                        .on_hover_text("Click to Stop and Reset recorded data buffer");
                                    match (
                                        btn_start.clicked(),
                                        btn_reset.clicked(),
                                        serial_open,
                                    ) {
                                        (true, false, false) => {
                                            control.service.start_serial();
                                        }
                                        (false, true, true) => {
                                            control.service.stop_serial();
                                            control.buffer.clean();
                                        }
                                        (false, true, false) => control.buffer.clean(),
                                        _ => {}
                                    }
                                }
                                None => {
                                    Label::new(RichText::new("STATUS: Not Initialize / Connected").color(Color32::RED))
                                        .sense(Sense::union(Sense::click(), Sense::hover()))
                                        .ui(scr_ui)
                                        .on_hover_text(
                                            "PORT INFO: [NO PORT DETECTED] (XX:XX), click to try Initialize the port",
                                        );
                                    if scr_ui.button("\u{1F50C} Try Reconnect").clicked() {
                                        control.service.reconnect_serial();
                                    }
                                }
                            }
                            scr_ui.separator();
                            Self::setting_info(scr_ui, &mut control.config);
                        }
                        PanelSetting::Style => {
                            ctx.settings_ui(scr_ui);
                            scr_ui.separator();
                            ctx.inspection_ui(scr_ui);
                        }
                    };
                    scr_ui.add_space(20.);
                };

                ScrollArea::vertical()
                    .id_source("dyno_settings")
                    .show(ui, scroll_area_ui);
            });

        self.open = open;
        self.edit_path = edit_path;
        self.panel = panel;
    }

    #[inline]
    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.open
    }
}
