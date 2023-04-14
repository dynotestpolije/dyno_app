use std::path::{Path, PathBuf};

use dyno_types::{
    data_buffer::{BufferData, Data},
    infomotor::{self, InfoMotor},
    DynoResult,
};
use dynotest_app::{
    config::CoreConfig,
    paths::DynoPaths,
    service::{self, PortInfo, SerialService},
    widgets::{
        button::{ButtonExt, ButtonKind},
        DynoFileManager, DynoWidgets, MultiRealtimePlot,
    },
};
use eframe::egui::*;

use crate::condition::FileType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
enum PanelSetting {
    #[default]
    Generic,
    InfoMotor,
    Style,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DynoControl {
    #[serde(skip)]
    #[serde(default)]
    service: Option<SerialService>,

    #[serde(skip)]
    #[serde(default)]
    buffer: BufferData,

    info: InfoMotor,
    panel_setting: PanelSetting,
    edit_path: bool,
    show_setting_ui: bool,
}
impl Default for DynoControl {
    fn default() -> Self {
        Self {
            service: None,
            buffer: BufferData::new(),
            info: InfoMotor::new(),
            panel_setting: PanelSetting::Generic,
            edit_path: false,
            show_setting_ui: false,
        }
    }
}

impl DynoControl {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn init_serial(&mut self) {
        self.service = service::init_serial();
    }

    #[inline]
    pub fn on_pos_render(&mut self) {
        if let Some(Ok(serial_data)) = self.service.as_mut().map(|x| x.handle()) {
            let data = Data::from_serial(&self.info, serial_data);
            self.buffer.push_data(data);
        }
    }

    #[inline(always)]
    pub fn last_buffer(&self) -> Data {
        self.buffer.last()
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn buffer(&self) -> &'_ BufferData {
        &self.buffer
    }
    #[inline(always)]
    pub fn buffer_mut(&mut self) -> &'_ mut BufferData {
        &mut self.buffer
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn info_motor(&self) -> &'_ InfoMotor {
        &self.info
    }
}

impl DynoControl {
    #[inline]
    #[allow(unused)]
    fn setting_generic(
        &mut self,
        ui: &mut Ui,
        paths: &mut DynoPaths,
        config: &mut CoreConfig,
    ) -> Response {
        ScrollArea::vertical()
            .id_source("dyno_setting_info")
            .show(ui, |scr_ui| {
                let resp = CollapsingHeader::new("✒ Paths")
                    .default_open(true)
                    .show(scr_ui, |ui| {
                        ui.add(TextEdit::singleline(&mut paths.name).hint_text("app dir name"));
                        ui.separator();
                        ui.checkbox(&mut self.edit_path, "Edit Paths Config");
                        paths.draw(ui, self.edit_path);
                    })
                    .header_response;
                scr_ui.separator();
                resp.union(
                    CollapsingHeader::new(" Configurations")
                        .default_open(true)
                        .show(scr_ui, |ui| {
                            ui.checkbox(&mut config.show_startup, "Show Startup Window");
                            ui.separator();
                            config.app_options.ui(ui);
                        })
                        .header_response,
                )
            })
            .inner
    }

    #[inline]
    #[allow(unused)]
    fn setting_info(&mut self, ui: &mut Ui) -> Response {
        ScrollArea::vertical()
            .id_source("dyno_setting_info")
            .show(ui, |scr_ui| {
                let resp = scr_ui.heading("Info Setting");
                scr_ui.separator();
                scr_ui.horizontal_wrapped(|horzui| {
                    horzui
                        .add(TextEdit::singleline(&mut self.info.name).hint_text("isi nama motor"));
                    horzui.separator();
                    horzui.add(
                        DragValue::new(&mut self.info.cc)
                            .speed(1)
                            .prefix("Volume Cilinder: ")
                            .suffix(" cc")
                            .min_decimals(10)
                            .max_decimals(30),
                    );
                });
                scr_ui.separator();
                scr_ui.horizontal_wrapped(|horzui| {
                    horzui.selectable_value_from_iter(
                        &mut self.info.cylinder,
                        infomotor::Cylinder::into_iter(),
                    );
                    horzui.separator();
                    horzui.selectable_value_from_iter(
                        &mut self.info.stroke,
                        infomotor::Stroke::into_iter(),
                    );
                    horzui.separator();
                    horzui.selectable_value_from_iter(
                        &mut self.info.transmition,
                        infomotor::Transmition::into_iter(),
                    );
                });
                scr_ui.separator();
                scr_ui.add(
                    DragValue::new(&mut self.info.tire_diameter)
                        .speed(1)
                        .prefix("Diameter Ban: ")
                        .suffix(" inch")
                        .min_decimals(10)
                        .max_decimals(50),
                );
                resp
            })
            .inner
    }

    #[allow(unused)]
    pub fn show_setting(
        &mut self,
        ctx: &Context,
        show: &mut bool,
        paths: &mut DynoPaths,
        config: &mut CoreConfig,
    ) {
        let setting_window = |ui: &mut Ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    use PanelSetting::*;
                    ui.selectable_value(&mut self.panel_setting, Generic, stringify!(Generic));
                    ui.selectable_value(&mut self.panel_setting, InfoMotor, stringify!(InfoMotor));
                    ui.selectable_value(&mut self.panel_setting, Style, stringify!(Style));
                });
            });
            ui.separator();
            match self.panel_setting {
                PanelSetting::Generic => self.setting_generic(ui, paths, config),
                PanelSetting::InfoMotor => self.setting_info(ui),
                PanelSetting::Style => {
                    ScrollArea::vertical()
                        .id_source("dyno_setting_style")
                        .show(ui, |scr_ui| {
                            ctx.settings_ui(scr_ui);
                            let resp = scr_ui.separator();
                            ctx.inspection_ui(scr_ui);
                            resp
                        })
                        .inner
                }
            }
        };
        Window::new("Dyno Control Settings")
            .id(Id::new("id_control_setting"))
            .open(show)
            .collapsible(false)
            .resizable(true)
            .show(ctx, setting_window);
    }
}

// ----------- DRAWER --------------
impl DynoControl {
    pub fn show_status(&self, ui: &mut Ui) {
        match self.service {
            Some(ref serial) => {
                let PortInfo {
                    port_name,
                    vid,
                    pid,
                    ..
                } = serial.get_info();
                ui.colored_label(Color32::DARK_GREEN, "Running");
                ui.separator();
                ui.small(format!(
                    "connected `PORT`[VID:PID]: `{port_name}`[{vid}:{pid}]"
                ));
                ui.separator();
                ui.small_stop_button()
                    .on_hover_text("Click to Stop the Service");
            }
            None => {
                ui.colored_label(Color32::DARK_RED, "Not Running");
                ui.separator();
                ui.small("connected `PORT`[VID:PID]: `NO PORT`[00:00]");
                ui.separator();
                ui.small_play_button()
                    .on_hover_text("Click to Start the Service");
            }
        }
        ui.separator();
        ui.small(format!("Active Info: {}", self.info));
    }
}

impl DynoControl {
    #[allow(unused)]
    #[inline(always)]
    pub fn show_plot(&self, ui: &mut Ui) -> Response {
        MultiRealtimePlot::new().animate(true).ui(ui, &self.buffer)
    }
}

impl DynoControl {
    #[inline]
    fn saves(&mut self, types: FileType, file: PathBuf) -> DynoResult<Option<PathBuf>> {
        match types {
            FileType::Binaries => self.buffer.serialize_to_file(&file)?,
            FileType::Csv => self.buffer.save_as_csv(&file)?,
            FileType::Excel => self.buffer.save_as_excel(&file)?,
            _ => (),
        };
        Ok(Some(file))
    }
    pub fn on_save(
        &mut self,
        tp: FileType,
        dirpath: impl AsRef<Path>,
    ) -> DynoResult<Option<PathBuf>> {
        let dirpath = dirpath.as_ref();
        match tp {
            FileType::All => match DynoFileManager::pick_all_type(dirpath) {
                Some(file) => match file.extension().map(|osstr| osstr.to_str().unwrap_or("")) {
                    Some("bin") | Some("dbin") => self.saves(FileType::Binaries, file),
                    Some("csv") | Some("dynocsv") => self.saves(FileType::Csv, file),
                    Some("xlsx") => self.saves(FileType::Excel, file),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            FileType::Binaries => match DynoFileManager::pick_binaries(dirpath) {
                Some(file) => self.saves(tp, file),
                None => Ok(None),
            },
            FileType::Csv => match DynoFileManager::pick_csv(dirpath) {
                Some(file) => self.saves(tp, file),
                None => Ok(None),
            },
            FileType::Excel => match DynoFileManager::pick_excel(dirpath) {
                Some(file) => self.saves(tp, file),
                None => Ok(None),
            },
        }
    }

    fn opens(&mut self, types: FileType, file: PathBuf) -> DynoResult<Option<PathBuf>> {
        self.buffer.clean();
        self.buffer = match types {
            FileType::Binaries => BufferData::deserialize_from_file(&file)?,
            FileType::Csv => BufferData::open_from_csv(&file)?,
            FileType::Excel => BufferData::open_from_excel(&file)?,
            _ => return Ok(None),
        };
        Ok(Some(file))
    }
    pub fn on_open(
        &mut self,
        tp: FileType,
        dirpath: impl AsRef<Path>,
    ) -> DynoResult<Option<PathBuf>> {
        let dirpath = dirpath.as_ref();
        match tp {
            FileType::All => match DynoFileManager::pick_all_type(dirpath) {
                Some(file) => match file.extension().map(|osstr| osstr.to_str().unwrap_or("")) {
                    Some("bin") | Some("dbin") => self.opens(FileType::Binaries, file),
                    Some("csv") | Some("dynocsv") => self.opens(FileType::Csv, file),
                    Some("xlsx") => self.opens(FileType::Excel, file),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            FileType::Binaries => match DynoFileManager::pick_binaries(dirpath) {
                Some(file) => self.opens(tp, file),
                None => Ok(None),
            },
            FileType::Csv => match DynoFileManager::pick_csv(dirpath) {
                Some(file) => self.opens(tp, file),
                None => Ok(None),
            },
            FileType::Excel => match DynoFileManager::pick_excel(dirpath) {
                Some(file) => self.opens(tp, file),
                None => Ok(None),
            },
        }
    }

    #[inline]
    pub fn popup_unsaved() -> ButtonKind {
        if !dynotest_app::msg_dialog_warn!(
            OkCancel => ["Save the project", "Cancel the Warning"],
            "Unsaved Project Buffer",
            "cause: unsave project buffer"
        ) {
            return ButtonKind::Cancel;
        }
        ButtonKind::Ok
    }
}
impl AsRef<DynoControl> for DynoControl {
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}
