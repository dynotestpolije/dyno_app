use crate::{
    config::ApplicationConfig,
    paths::DynoPaths,
    service::{init_serial, PortInfo, SerialService},
    state::{DynoFileType, DynoState, OperatorData},
    toast_error, toast_success,
    widgets::{button::ButtonExt, DynoFileManager, MultiRealtimePlot},
};
use dyno_core::{
    chrono::{NaiveDateTime, Utc},
    serde, BufferData, CompresedSaver, Data, DynoConfig, DynoResult,
};
use eframe::egui::*;
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(crate = "serde")]
enum PanelSetting {
    #[default]
    Generic,
    Config,
    Style,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub struct DynoControl {
    pub paths: DynoPaths,
    pub app_config: ApplicationConfig,
    pub config: DynoConfig,

    plots: MultiRealtimePlot,

    #[serde(skip)]
    #[serde(default)]
    service: Option<SerialService>,

    #[serde(skip)]
    #[serde(default)]
    buffer: BufferData,

    panel_setting: PanelSetting,
    edit_path: bool,
    buffer_saved: bool,

    start: Option<NaiveDateTime>,
}
impl Default for DynoControl {
    fn default() -> Self {
        let paths = DynoPaths::new(crate::PACKAGE_INFO.app_name).unwrap_or_else(|err| {
            toast_error!("Failed to get Paths Information in this Device ({err})");
            Default::default()
        });

        let app_config = paths
            .get_config::<ApplicationConfig>("app_config.toml")
            .unwrap_or_else(|err| {
                toast_error!("Failed to get Application Configuration file ({err})");
                Default::default()
            });

        let config = paths
            .get_config::<DynoConfig>("config.toml")
            .unwrap_or_else(|err| {
                toast_error!("Failed to get DynoTests Configuration file ({err})");
                Default::default()
            });

        Self {
            app_config,
            config,
            paths,
            service: None,
            buffer: BufferData::new(),
            panel_setting: PanelSetting::Generic,
            plots: MultiRealtimePlot::new(),
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
        if let Some(Some(serial_data)) = self.service.as_ref().map(|x| x.handle()) {
            self.buffer.push_from_serial(&self.config, serial_data);
            self.buffer_saved = false;
        }
    }

    #[inline(always)]
    pub fn last_buffer(&self) -> Data {
        self.buffer.last().clone()
    }

    pub fn service_start(&mut self) -> DynoResult<()> {
        if let Some(ref mut serial) = self.service {
            self.start = Some(Utc::now().naive_local());
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
    #[inline(always)]
    pub fn show_plot(&mut self, ui: &mut Ui) -> Response {
        self.plots.ui(ui, &self.buffer)
    }
}

impl DynoControl {
    #[inline]
    fn saves(&mut self, types: DynoFileType, file: PathBuf) -> DynoResult<Option<PathBuf>> {
        match types {
            DynoFileType::Dyno => self.buffer.compress_to_file(&file)?,
            DynoFileType::Csv => self.buffer.save_as_csv(&file)?,
            DynoFileType::Excel => self.buffer.save_as_excel(&file)?,
            _ => (),
        };
        self.buffer_saved = true;
        Ok(Some(file))
    }
    pub fn on_save(&mut self, tp: DynoFileType) -> DynoResult<Option<PathBuf>> {
        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
        match tp {
            DynoFileType::All => match DynoFileManager::pick_all_type(dirpath) {
                Some(file) => match file.extension().map(|osstr| osstr.to_str().unwrap_or("")) {
                    Some("dyno") | Some("dbin") => self.saves(DynoFileType::Dyno, file),
                    Some("csv") | Some("dynocsv") => self.saves(DynoFileType::Csv, file),
                    Some("xlsx") => self.saves(DynoFileType::Excel, file),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            DynoFileType::Dyno => match DynoFileManager::pick_binaries(dirpath) {
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

    #[inline]
    pub fn handle_states(
        &mut self,
        ctx: &Context,
        frame: &mut eframe::Frame,
        state: &mut DynoState,
    ) {
        match (state.get_operator(), self.is_buffer_saved()) {
            // if buffer is saved and operator want to save, do save the buffer, or if buffer
            // already saved, ignore the operator
            (OperatorData::SaveFile(tp), false) => {
                match self.on_save(tp) {
                    Ok(Some(path)) => {
                        toast_success!("Saved {tp} file: {}", path.display())
                    }
                    Err(err) => toast_error!("Error on Saving {tp}: {err}"),
                    _ => {}
                };
                if state.quitable() {
                    state.set_quit(true);
                }
            }
            // if buffer is saved and operator want to open file, do open the file to buffer,
            // or is buffer unsaved but operator want ot open file, show popup to save buffer first
            (OperatorData::OpenFile(tp), true) => match self.on_open(tp) {
                Ok(Some(p)) => toast_success!("Opened {tp} file: {}", p.display()),
                Err(err) => toast_error!("Failed Opening File: {err}"),
                _ => {}
            },
            (OperatorData::OpenFile(_), false) => state.set_show_buffer_unsaved(true),
            _ => {}
        }
        if state.global_loading() {
            ctx.layer_painter(LayerId::new(
                Order::Background,
                Id::new("confirmation_popup_unsaved"),
            ))
            .rect_filled(
                ctx.input(|inp| inp.screen_rect()),
                0.0,
                Color32::from_black_alpha(192),
            );
            Area::new("dyno_global_loading_spinner")
                .order(Order::Foreground)
                .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.add(
                        Spinner::new()
                            .color(crate::COLOR_BLUE_DYNO)
                            .size(ctx.available_rect().height() / 2.),
                    )
                });
        }

        if state.quit() {
            frame.close();
        }
    }

    fn opens(&mut self, types: DynoFileType, file: PathBuf) -> DynoResult<Option<PathBuf>> {
        self.buffer.clean();
        self.buffer = match types {
            DynoFileType::Dyno => BufferData::decompress_from_file(&file)?,
            DynoFileType::Csv => BufferData::open_from_csv(&file)?,
            DynoFileType::Excel => BufferData::open_from_excel(&file)?,
            _ => return Ok(None),
        };
        Ok(Some(file))
    }

    pub fn on_open(&mut self, tp: DynoFileType) -> DynoResult<Option<PathBuf>> {
        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
        match tp {
            DynoFileType::All => match DynoFileManager::pick_all_type(dirpath) {
                Some(file) => match file.extension().map(|osstr| osstr.to_str().unwrap_or("")) {
                    Some("bin") | Some("dbin") => self.opens(DynoFileType::Dyno, file),
                    Some("csv") | Some("dynocsv") => self.opens(DynoFileType::Csv, file),
                    Some("xlsx") => self.opens(DynoFileType::Excel, file),
                    _ => Ok(None),
                },
                None => Ok(None),
            },
            DynoFileType::Dyno => match DynoFileManager::pick_binaries(dirpath) {
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

impl DynoControl {
    pub fn bottom_status(&mut self, vertui: &mut Ui) {
        let layout_ui_status = |ltr_ui: &mut Ui| {
            match &self.service {
                Some(serial) => {
                    let PortInfo {
                        port_name,
                        vid,
                        pid,
                        ..
                    }: &PortInfo = serial.get_info();
                    let (status, color) = if serial.is_open() {
                        ("STATUS: Running", Color32::YELLOW)
                    } else {
                        ("STATUS: Connected", Color32::GREEN)
                    };
                    Label::new(RichText::new(status).color(color))
                        .ui(ltr_ui)
                        .on_hover_text(format!("PORT INFO: [{port_name}] ({vid}:{pid})"));
                }
                None => {
                    if Label::new(
                        RichText::new("STATUS: Not Initialize / Connected").color(Color32::RED),
                    )
                    .sense(Sense::union(Sense::click(), Sense::hover()))
                    .ui(ltr_ui)
                    .on_hover_text(
                        "PORT INFO: [NO PORT DETECTED] (XX:XX), click to try Initialize the port",
                    )
                    .clicked()
                    {
                        self.service = match SerialService::new() {
                            Ok(serial) => {
                                toast_success!(
                                    "SUCCES! connected to [{}] - [{}:{}]",
                                    serial.info.port_name,
                                    serial.info.vid,
                                    serial.info.pid
                                );
                                Some(serial)
                            }
                            Err(err) => {
                                toast_error!("Failed to Reinitialize Serial Service - ({err})");
                                None
                            }
                        };
                    }
                }
            }
            ltr_ui.separator();
            if ltr_ui
                .small_play_button()
                .on_hover_text("Click to Start the Service")
                .clicked()
            {
                if let Err(err) = self.service_start() {
                    toast_error!("ERROR: Failed to start Serial Service - ({err})");
                }
            }
            if ltr_ui
                .small_stop_button()
                .on_hover_text("Click to Stop/Pause the Service")
                .clicked()
            {
                if let Some(serial) = &mut self.service {
                    serial.stop();
                } else {
                    toast_error!(
                        "[ERROR] Serial Port is not Initialize or not Connected!,\
                            try to Click on bottom left on 'STATUS' to reinitialize or reconnected",
                    );
                }
            }
            if ltr_ui
                .small_reset_button()
                .on_hover_text("Click to Reset recorded data buffer")
                .clicked()
            {
                if let Some(serial) = &mut self.service {
                    serial.stop();
                    self.buffer.clean();
                } else {
                    toast_error!(
                        "[ERROR] Serial Port is not Initialize or not Connected!,\
                            try to Click on bottom left on 'STATUS' to reinitialize or reconnected",
                    );
                }
            }
        };
        vertui.with_layout(Layout::left_to_right(Align::Center), layout_ui_status);
        vertui.separator();
        vertui.with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
            rtl_ui.small(format!("Active Info: {}", self.config.motor_type));
        });
    }
}

impl AsRef<DynoControl> for DynoControl {
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}
