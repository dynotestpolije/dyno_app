use std::path::PathBuf;

use dyno_types::{
    data_buffer::{BufferData, Data},
    infomotor::{self, InfoMotor},
    DynoResult,
};
use dynotest_app::{
    config::CoreConfig,
    paths::DynoPaths,
    service::{PortInfo, SerialService},
    widgets::{
        button::{ButtonExt, ButtonKind},
        popup::{PopupLevel, PopupWindow},
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
pub struct Controller {
    #[serde(skip)]
    #[serde(default)]
    service: Option<SerialService>,

    paths: DynoPaths,
    config: CoreConfig,

    buffer: BufferData,
    info: InfoMotor,
    panel_setting: PanelSetting,
    edit_path: bool,

    show_setting_ui: bool,
}
impl Default for Controller {
    fn default() -> Self {
        let service = dynotest_app::service::init_serial();
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
        let config = paths
            .get_config::<CoreConfig>("config.toml")
            .unwrap_or_default();
        Self {
            service,
            paths,
            config,
            buffer: BufferData::new(),
            info: InfoMotor::new(),
            panel_setting: PanelSetting::Generic,
            edit_path: false,
            show_setting_ui: false,
        }
    }
}

impl Controller {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn check_if_change(&mut self, other: impl AsRef<Self>) {
        let Self { config, paths, .. } = other.as_ref();
        self.config.check_is_changed(config);
        self.paths.check_is_changed(paths);
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

    #[inline(always)]
    pub fn buffer(&self) -> &'_ BufferData {
        &self.buffer
    }
    #[inline(always)]
    pub fn buffer_mut(&mut self) -> &'_ mut BufferData {
        &mut self.buffer
    }

    #[inline(always)]
    pub fn info_motor(&self) -> &'_ InfoMotor {
        &self.info
    }
    #[inline(always)]
    pub fn paths(&self) -> &'_ DynoPaths {
        &self.paths
    }
    #[inline(always)]
    #[allow(unused)]
    pub fn config(&self) -> &'_ CoreConfig {
        &self.config
    }
    #[inline(always)]
    pub fn app_options(&self) -> eframe::NativeOptions {
        self.config.app_options.main_window_opt()
    }
    #[inline(always)]
    pub fn option_show_startup(&self) -> Option<eframe::NativeOptions> {
        if self.config.show_startup {
            return Some(self.config.app_options.startup_opt());
        }
        None
    }
}

impl Controller {
    #[inline]
    #[allow(unused)]
    fn setting_generic(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            CollapsingHeader::new("✒ Paths")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(TextEdit::singleline(&mut self.paths.name).hint_text("app dir name"));
                    ui.separator();
                    ui.checkbox(&mut self.edit_path, "Edit Paths Config");
                    self.paths.draw(ui, self.edit_path);
                });
            ui.separator();
            CollapsingHeader::new(" Configurations")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.config.show_startup, "Show Startup Window");
                    ui.separator();
                    self.config.app_options.ui(ui);
                });
        })
        .response
    }

    #[inline]
    #[allow(unused)]
    fn setting_info(&mut self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.heading("Info Setting");
            self.show_status(ui);
            ui.separator();
            ui.add(TextEdit::singleline(&mut self.info.name).hint_text("isi nama motor"));
            ui.separator();
            ui.add(
                DragValue::new(&mut self.info.cc)
                    .speed(1)
                    .prefix("Volume Cilinder: ")
                    .suffix(" cc")
                    .min_decimals(10)
                    .max_decimals(30),
            );
            ui.separator();
            ui.selectable_value_from_iter(
                &mut self.info.cylinder,
                infomotor::Cylinder::into_iter(),
            );
            ui.separator();
            ui.selectable_value_from_iter(&mut self.info.stroke, infomotor::Stroke::into_iter());
            ui.separator();
            ui.selectable_value_from_iter(
                &mut self.info.transmition,
                infomotor::Transmition::into_iter(),
            );
            ui.separator();
            ui.add(
                DragValue::new(&mut self.info.tire_diameter)
                    .speed(1)
                    .prefix("Diameter Ban: ")
                    .suffix(" inch")
                    .min_decimals(10)
                    .max_decimals(50),
            );
        })
        .response
    }

    #[allow(unused)]
    pub fn show_setting(&mut self, ctx: &Context, show: &mut bool) {
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
                PanelSetting::Generic => self.setting_generic(ui),
                PanelSetting::InfoMotor => self.setting_info(ui),
                PanelSetting::Style => {
                    ui.vertical_centered_justified(|ui| {
                        ctx.settings_ui(ui);
                        ui.separator();
                        ctx.inspection_ui(ui);
                    })
                    .response
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
impl Controller {
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
        ui.separator();
    }
}

impl Controller {
    #[allow(unused)]
    pub fn show_plot(&self, ui: &mut Ui) -> Response {
        MultiRealtimePlot::new().ui(ui, &self.buffer, &self.info)
    }
}

impl Controller {
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
    pub fn on_save(&mut self, tp: FileType) -> DynoResult<Option<PathBuf>> {
        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
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
    pub fn on_open(&mut self, tp: FileType) -> DynoResult<Option<PathBuf>> {
        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
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
        const POPUP_BUTTONS_UNSAVED_WARN: [(ButtonKind, &'_ str); 3] = [
            (ButtonKind::Ok, "click 'Ok' to save the unsaved data"),
            (ButtonKind::No, "click 'No', to ignore the warning"),
            (ButtonKind::Cancel, "click 'Cancel', to cancel"),
        ];
        PopupWindow::new(
            "Unsaved DynoTest!",
            "Do you want to save the Tests?\n",
            POPUP_BUTTONS_UNSAVED_WARN,
        )
        .set_level(PopupLevel::Warning)
        .show()
    }
}
impl AsRef<Controller> for Controller {
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}
