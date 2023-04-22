use crate::{
    config::DynoConfig,
    paths::DynoPaths,
    service::{self, PortInfo, SerialService},
    widgets::{button::ButtonExt, DynoFileManager, MultiRealtimePlot},
};
use dyno_types::{
    data_buffer::{BufferData, Data},
    infomotor::InfoMotor,
    DynoResult,
};
use eframe::egui::*;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::state::DynoFileType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
enum PanelSetting {
    #[default]
    Generic,
    InfoMotor,
    Style,
}

#[derive(Deserialize, Serialize)]
pub struct DynoControl {
    #[serde(skip)]
    #[serde(default)]
    service: Option<SerialService>,

    #[serde(skip)]
    #[serde(default)]
    buffer: BufferData,

    paths: Arc<RwLock<DynoPaths>>,
    config: Arc<RwLock<DynoConfig>>,
    info: Arc<RwLock<InfoMotor>>,
    panel_setting: PanelSetting,
    edit_path: bool,
}
impl Default for DynoControl {
    fn default() -> Self {
        let paths = match DynoPaths::new(crate::PACKAGE_INFO.app_name) {
            Ok(p) => p,
            Err(err) => {
                if !crate::msg_dialog_err!(
                    OkIgnore => ["Quit the Application", "Ignore the error and continue the Application"],
                    "Error Initializing Path Config",
                    "cause: {err}"
                ) {
                    dyno_types::log::error!("Quiting Application From error; {err}");
                    std::process::exit(0);
                }
                DynoPaths::default()
            }
        };
        let config = Arc::new(RwLock::new(
            paths
                .get_config::<DynoConfig>("config.toml")
                .unwrap_or_default(),
        ));
        Self {
            config,
            paths: Arc::new(RwLock::new(paths)),
            info: Arc::new(RwLock::new(InfoMotor::new())),
            service: None,
            buffer: BufferData::new(),
            panel_setting: PanelSetting::Generic,
            edit_path: false,
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
            let Ok(info) = self.info.read() else {
                return;
            };
            let data = Data::from_serial(&info, serial_data);
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

    #[inline(always)]
    pub fn buffer_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn info_motor(&self) -> &Arc<RwLock<InfoMotor>> {
        &self.info
    }

    #[inline]
    pub fn paths(&self) -> &Arc<RwLock<DynoPaths>> {
        &self.paths
    }
    #[inline]
    pub fn config(&self) -> &Arc<RwLock<DynoConfig>> {
        &self.config
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
        if let Ok(info) = self.info.read() {
            ui.small(format!("Active Info: {}", info));
        }
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
    fn saves(&mut self, types: DynoFileType, file: PathBuf) -> DynoResult<Option<PathBuf>> {
        match types {
            DynoFileType::Binaries => self.buffer.serialize_to_file(&file)?,
            DynoFileType::Csv => self.buffer.save_as_csv(&file)?,
            DynoFileType::Excel => self.buffer.save_as_excel(&file)?,
            _ => (),
        };
        Ok(Some(file))
    }
    pub fn on_save(&mut self, tp: DynoFileType) -> DynoResult<Option<PathBuf>> {
        let dirpath = {
            let Ok(path_manager) = self.paths.read() else {
                return Err(From::from("ERROR on reading/lock RwLock of path_manager"));
            };
            tp.path(path_manager.get_data_dir_folder("Saved"))
        };
        match tp {
            DynoFileType::All => match DynoFileManager::pick_all_type(dirpath) {
                Some(file) => match file.extension().map(|osstr| osstr.to_str().unwrap_or("")) {
                    Some("bin") | Some("dbin") => self.saves(DynoFileType::Binaries, file),
                    Some("csv") | Some("dynocsv") => self.saves(DynoFileType::Csv, file),
                    Some("xlsx") => self.saves(DynoFileType::Excel, file),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            DynoFileType::Binaries => match DynoFileManager::pick_binaries(dirpath) {
                Some(file) => self.saves(tp, file),
                None => Ok(None),
            },
            DynoFileType::Csv => match DynoFileManager::pick_csv(dirpath) {
                Some(file) => self.saves(tp, file),
                None => Ok(None),
            },
            DynoFileType::Excel => match DynoFileManager::pick_excel(dirpath) {
                Some(file) => self.saves(tp, file),
                None => Ok(None),
            },
        }
    }

    fn opens(&mut self, types: DynoFileType, file: PathBuf) -> DynoResult<Option<PathBuf>> {
        self.buffer.clean();
        self.buffer = match types {
            DynoFileType::Binaries => BufferData::deserialize_from_file(&file)?,
            DynoFileType::Csv => BufferData::open_from_csv(&file)?,
            DynoFileType::Excel => BufferData::open_from_excel(&file)?,
            _ => return Ok(None),
        };
        Ok(Some(file))
    }
    pub fn on_open(&mut self, tp: DynoFileType) -> DynoResult<Option<PathBuf>> {
        let dirpath = {
            let Ok(path_manager) = self.paths.read() else {
                return Err(From::from("ERROR on reading/lock RwLock of path_manager"));
            };
            tp.path(path_manager.get_data_dir_folder("Saved"))
        };
        match tp {
            DynoFileType::All => match DynoFileManager::pick_all_type(dirpath) {
                Some(file) => match file.extension().map(|osstr| osstr.to_str().unwrap_or("")) {
                    Some("bin") | Some("dbin") => self.opens(DynoFileType::Binaries, file),
                    Some("csv") | Some("dynocsv") => self.opens(DynoFileType::Csv, file),
                    Some("xlsx") => self.opens(DynoFileType::Excel, file),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            DynoFileType::Binaries => match DynoFileManager::pick_binaries(dirpath) {
                Some(file) => self.opens(tp, file),
                None => Ok(None),
            },
            DynoFileType::Csv => match DynoFileManager::pick_csv(dirpath) {
                Some(file) => self.opens(tp, file),
                None => Ok(None),
            },
            DynoFileType::Excel => match DynoFileManager::pick_excel(dirpath) {
                Some(file) => self.opens(tp, file),
                None => Ok(None),
            },
        }
    }
}
impl AsRef<DynoControl> for DynoControl {
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}
