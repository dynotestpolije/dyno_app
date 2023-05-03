use crate::{
    config::DynoConfig,
    paths::DynoPaths,
    service::{init_serial, SerialService},
    widgets::{DynoFileManager, MultiRealtimePlot},
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
    buffer_saved: bool,

    start: Option<dyno_types::chrono::NaiveDateTime>,
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
            buffer_saved: false,
            start: None,
        }
    }
}

impl DynoControl {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn init_serial(&mut self) {
        self.service = init_serial();
    }

    #[inline]
    pub fn on_pos_render(&mut self) {
        if let Some(Ok(serial_data)) = self.service.as_mut().map(|x| x.handle()) {
            let Ok(info) = self.info.read() else {
                return;
            };
            let data = Data::from_serial(&info, serial_data);
            self.buffer.push_data(data);
            self.buffer_saved = false;
        }
    }

    #[inline(always)]
    pub fn last_buffer(&self) -> Data {
        self.buffer.last()
    }

    pub fn service(&self) -> Option<&SerialService> {
        self.service.as_ref()
    }

    pub fn service_mut(&mut self) -> Option<&mut SerialService> {
        self.service.as_mut()
    }

    pub fn reinitialize_service(&mut self) -> DynoResult<()> {
        self.service = Some(SerialService::new()?);
        Ok(())
    }

    pub fn service_start(&mut self) -> DynoResult<'_, ()> {
        if let Some(ref mut serial) = self.service {
            self.start = Some(dyno_types::chrono::Utc::now().naive_local());
            serial.start()?;
            Ok(())
        } else {
            Err(From::from("[ERROR] Serial Port is not Initialize or not Connected!, try to Click on bottom left on 'STATUS' to reinitialize or reconnected"))
        }
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

    // mark return saved if buffer is already saved or buffer is empty
    pub fn is_buffer_saved(&self) -> bool {
        self.buffer_saved || self.buffer_empty()
    }

    #[inline]
    pub fn start_time(&self, data: &Data) -> String {
        if let Some(dt) = self.start {
            data.time_duration_formatted(dt.time())
        } else {
            "00:00:00".to_owned()
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
        self.buffer_saved = true;
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
