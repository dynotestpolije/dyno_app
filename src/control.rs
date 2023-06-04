use crate::{
    config::ApplicationConfig,
    paths::DynoPaths,
    row_label_value,
    service::{ApiService, PortInfo, SerialService},
    state::{DynoFileType, DynoState, OperatorData},
    toast_error, toast_info, toast_success,
    widgets::{
        button::ButtonExt, segment_display::SegmentedDisplay, DynoFileManager, Gauge, RealtimePlot,
    },
    AsyncMsg,
};
use dyno_core::{
    asyncify,
    chrono::{NaiveDateTime, Utc},
    crossbeam_channel::{unbounded, Receiver, Sender},
    ignore_err, serde, BufferData, CompresedSaver, CsvSaver, Data, DynoConfig, DynoResult,
    ExcelSaver,
};
use eframe::egui::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[cfg_attr(debug_assertions, derive(Debug))]
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

    #[serde(skip)]
    #[serde(default)]
    buffer: BufferData,

    #[serde(skip)]
    #[serde(default)]
    serial_service: Option<SerialService>,

    #[serde(skip)]
    #[serde(default)]
    api_service: Option<ApiService>,

    plots: RealtimePlot,

    #[serde(skip)]
    #[serde(default = "unbounded")]
    async_channels: (Sender<AsyncMsg>, Receiver<AsyncMsg>),

    #[serde(skip)]
    start_time: u64,

    #[serde(skip)]
    pub start: Option<NaiveDateTime>,

    #[serde(skip)]
    pub stop: Option<NaiveDateTime>,

    #[serde(skip)]
    #[serde(default)]
    loadings: Arc<AtomicBool>,

    #[serde(skip)]
    #[serde(default)]
    buffer_saved: bool,
}

impl Default for DynoControl {
    fn default() -> Self {
        Self::new()
    }
}

impl DynoControl {
    pub fn new() -> Self {
        let paths = DynoPaths::new(crate::PACKAGE_INFO.app_name).unwrap_or_else(|err| {
            dyno_core::log::error!("{err}");
            Default::default()
        });

        let app_config = paths
            .get_config::<ApplicationConfig>("app_config.toml")
            .unwrap_or_else(|err| {
                dyno_core::log::error!("{err}");
                Default::default()
            });

        let config = paths
            .get_config::<DynoConfig>("config.toml")
            .unwrap_or_else(|err| {
                dyno_core::log::error!("Failed to get DynoTests Configuration file ({err})");
                Default::default()
            });

        Self {
            app_config,
            config,
            paths,
            buffer: BufferData::new(),
            plots: RealtimePlot::new(),
            buffer_saved: true,
            async_channels: unbounded(),
            api_service: ApiService::new().map_or_else(
                |err| {
                    toast_error!("{err}");
                    None
                },
                Some,
            ),
            serial_service: Default::default(),
            start_time: Default::default(),
            start: Default::default(),
            stop: Default::default(),
            loadings: Default::default(),
        }
    }

    #[inline(always)]
    pub fn last_buffer(&self) -> Data {
        self.buffer.last().clone()
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

    #[inline]
    pub fn start_time(&self) -> String {
        let seconds = (self.start_time / 1000) % 60;
        let minutes = (self.start_time / (1000 * 60)) % 60;
        let hours = (self.start_time / (1000 * 60 * 60)) % 24;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    // mark return saved if buffer is already saved or buffer is empty
    pub const fn is_buffer_saved(&self) -> bool {
        self.buffer_saved || self.buffer.is_empty()
    }

    #[inline]
    pub fn tx(&self) -> &Sender<AsyncMsg> {
        &self.async_channels.0
    }
    #[inline]
    pub fn rx(&self) -> &Receiver<AsyncMsg> {
        &self.async_channels.1
    }
    #[inline]
    pub fn api(&self) -> Option<&ApiService> {
        self.api_service.as_ref()
    }
    #[inline]
    pub fn reconnect_api(&mut self) -> DynoResult<()> {
        self.api_service = Some(ApiService::new()?);
        Ok(())
    }
}

impl DynoControl {
    #[inline]
    pub fn on_pos_render(&mut self, state: &mut DynoState) {
        if let Ok(msg) = self.async_channels.1.try_recv() {
            match msg {
                AsyncMsg::OnSerialData(serial_data) => {
                    self.start_time += serial_data.period as u64;
                    self.buffer.push_from_serial(&self.config, serial_data);
                    self.buffer_saved = false;
                }
                AsyncMsg::OnOpenBuffer(buffer) => {
                    self.buffer = *buffer;
                    self.buffer_saved = false;
                }
                AsyncMsg::OnError(err) => {
                    toast_error!("ERROR HAS OCCURRED - ({err})")
                }
                AsyncMsg::OnSavedBuffer(()) => {
                    self.buffer_saved = true;
                    if state.quitable() {
                        state.set_quit(true);
                    }
                }
                AsyncMsg::OnCheckHealthApi(s) => {
                    if s.is_success() {
                        toast_success!("API Check Health is Success");
                    }
                }
                AsyncMsg::OnMessage(msg) => toast_info!("{msg}"),
            }
        }

        match (state.get_operator(), self.is_buffer_saved()) {
            // if buffer is saved and operator want to save, do save the buffer, or if buffer
            // already saved, ignore the operator
            (OperatorData::SaveFile(tp), false) => self.on_save(tp),
            // if buffer is saved and operator want to open file, do open the file to buffer,
            // or is buffer unsaved but operator want ot open file, show popup to save buffer first
            (OperatorData::OpenFile(tp), true) => self.on_open(tp),
            (OperatorData::OpenFile(_), false) => state.set_show_buffer_unsaved(true),
            _ => {}
        }
    }
    pub fn on_save(&mut self, tp: DynoFileType) {
        use dyno_core::tokio;

        let buffer = self.buffer.clone();
        let loadings = self.loadings.clone();
        let tx = self.async_channels.0.clone();

        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
        tokio::spawn(async move {
            loadings.store(true, Ordering::Relaxed);
            match tp {
                DynoFileType::Dyno => match DynoFileManager::pick_binaries_async(dirpath).await {
                    Some(file) => match asyncify!(move || buffer.compress_to_path(file.path())) {
                        Ok(()) => ignore_err!(tx.send(AsyncMsg::OnSavedBuffer(()))),
                        Err(err) => ignore_err!(tx.send(AsyncMsg::OnError(err))),
                    },
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
                DynoFileType::Csv => match DynoFileManager::pick_csv_async(dirpath).await {
                    Some(file) => match asyncify!(move || buffer.save_csv_from_path(file.path())) {
                        Ok(()) => ignore_err!(tx.send(AsyncMsg::OnSavedBuffer(()))),
                        Err(err) => ignore_err!(tx.send(AsyncMsg::OnError(err))),
                    },
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
                DynoFileType::Excel => match DynoFileManager::pick_excel_async(dirpath).await {
                    Some(file) => match asyncify!(move || buffer.save_excel_from_path(file.path()))
                    {
                        Ok(()) => ignore_err!(tx.send(AsyncMsg::OnSavedBuffer(()))),
                        Err(err) => ignore_err!(tx.send(AsyncMsg::OnError(err))),
                    },
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
            }
            loadings.store(false, Ordering::Relaxed);
        });
    }

    pub fn on_open(&mut self, tp: DynoFileType) {
        use dyno_core::tokio;

        let tx = self.async_channels.0.clone();
        let loadings = self.loadings.clone();
        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));

        tokio::spawn(async move {
            loadings.store(true, Ordering::Relaxed);
            match tp {
                DynoFileType::Dyno => match DynoFileManager::pick_binaries_async(dirpath).await {
                    Some(file) => {
                        match asyncify!(move || BufferData::decompress_from_path(file.path())) {
                            Ok(data) => ignore_err!(tx.send(AsyncMsg::open_buffer(data))),
                            Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                        }
                    }
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
                DynoFileType::Csv => match DynoFileManager::pick_csv_async(dirpath).await {
                    Some(file) => {
                        match asyncify!(move || BufferData::open_csv_from_path(file.path())) {
                            Ok(data) => ignore_err!(tx.send(AsyncMsg::open_buffer(data))),
                            Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                        }
                    }
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
                DynoFileType::Excel => match DynoFileManager::pick_excel_async(dirpath).await {
                    Some(file) => {
                        match asyncify!(move || BufferData::open_excel_from_path(file.path())) {
                            Ok(data) => ignore_err!(tx.send(AsyncMsg::open_buffer(data))),
                            Err(err) => ignore_err!(tx.send(AsyncMsg::error(err))),
                        }
                    }
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
            }
            loadings.store(false, Ordering::Relaxed);
        });
    }
}

impl DynoControl {
    #[inline(always)]
    pub fn bottom_status(&mut self, ui: &mut Ui) {
        let layout_ui_status = |ltr_ui: &mut Ui| match &mut self.serial_service {
            Some(serial) => {
                let (status, color) = if serial.is_open() {
                    ("STATUS: Running", Color32::YELLOW)
                } else {
                    ("STATUS: Connected", Color32::GREEN)
                };
                let PortInfo {
                    port_name,
                    vid,
                    pid,
                    ..
                }: &PortInfo = serial.get_info();
                Label::new(RichText::new(status).color(color))
                    .ui(ltr_ui)
                    .on_hover_text(format!("PORT INFO: [{port_name}] ({vid}:{pid})"));
                ltr_ui.separator();
                let btn_start = ltr_ui
                    .small_play_button()
                    .on_hover_text("Click to Start the Service");
                let btn_stop = ltr_ui
                    .small_stop_button()
                    .on_hover_text("Click to Stop/Pause the Service");
                let btn_reset = ltr_ui
                    .small_reset_button()
                    .on_hover_text("Click to Reset recorded data buffer");
                match (
                    btn_start.clicked(),
                    btn_stop.clicked(),
                    btn_reset.clicked(),
                    serial.is_open(),
                ) {
                    (true, _, _, false) => {
                        if let Err(err) = serial.start(self.async_channels.0.clone()) {
                            toast_error!("Serial Service Failed to start - {err}")
                        }
                        self.start = Some(Utc::now().naive_utc());
                    }
                    (_, true, _, true) => {
                        serial.stop();
                        self.stop = Some(Utc::now().naive_utc());
                    }
                    (_, _, true, true) => {
                        serial.stop();
                        self.buffer.clean();
                    }
                    _ => (),
                }
            }
            None => {
                Label::new(RichText::new("STATUS: Not Initialize / Connected").color(Color32::RED))
                    .sense(Sense::union(Sense::click(), Sense::hover()))
                    .ui(ltr_ui)
                    .on_hover_text(
                        "PORT INFO: [NO PORT DETECTED] (XX:XX), click to try Initialize the port",
                    );
                if ltr_ui.button("\u{1F50C} Try Reconnect").clicked() {
                    self.serial_service = match SerialService::new() {
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
                            toast_error!("Failed to Start Serial - ({err})");
                            None
                        }
                    };
                }
            }
        };
        ui.with_layout(Layout::left_to_right(Align::Center), layout_ui_status);
        ui.separator();
        ui.with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
            rtl_ui.small(format!("Active Info: {}", self.config.motor_type));
        });
    }

    pub fn left_panel(&mut self, ui: &mut Ui) {
        let Data {
            speed,
            torque,
            horsepower,
            rpm_roda,
            rpm_engine,
            temp,
            odo,
            percepatan_sudut,
            percepatan_roller,
            ..
        } = self.buffer.last();

        let grid_ui = |grid_ui: &mut Ui| {
            row_label_value!(
                grid_ui,
                speed,
                "Speed",
                "calculated from rotational speed distance of the roller in dynotests chasis"
            );
            row_label_value!(
                grid_ui,
                rpm_engine,
                "Rpm Engine",
                "calculated from rpm counter driver in dynotests chasis",
            );
            grid_ui.end_row();
            row_label_value!(
                grid_ui,
                rpm_engine,
                "Rpm ENgine (Mesin)",
                "calculated from rotational engine from rpm driver sensor in dynotests chasis",
            );
            row_label_value!(
                grid_ui,
                rpm_roda,
                "Rpm Tire (Roda)",
                "calculated from rotational of the roller in dynotests",
            );
            grid_ui.end_row();
            row_label_value!(
                grid_ui,
                torque,
                "Torque",
                "calculated from rotational speed of the roller in dynotests chasis",
            );
            row_label_value!(
                grid_ui,
                horsepower,
                "HorsePower",
                "calculated from rotational speed of the roller in dynotests chasis",
            );
            grid_ui.end_row();
            row_label_value!(
                grid_ui,
                odo,
                "ODO (Jarak Tempuh)",
                r#"
Distance Traveled / Jarak Tempuh calculated distance 
from rotational rounds of the roller in dynotests chasis
                "#,
            );
            row_label_value!(
                grid_ui,
                temp,
                "Engine Temperature",
                "calculated from thermocouple sensor driver in dynotests chasis",
            );
            grid_ui.end_row();
            row_label_value!(
                grid_ui,
                percepatan_sudut,
                "Angular Velocity",
                "calculated from angular rotational roller in dynotests chasis",
            );
            row_label_value!(
                grid_ui,
                percepatan_roller,
                "Roller Velocity",
                "calculated Roller velocity in dynotests chasis",
            );
        };
        ui.columns(2, |uis| {
            uis[0].add(Gauge::speed(*speed).diameter(uis[0].available_width()));
            uis[1].add(Gauge::rpm_engine(*rpm_engine).diameter(uis[1].available_width()));
        });
        CollapsingHeader::new("Gauges Other")
            .id_source("dyno_gauges_other_collapse_id")
            .show(ui, |ui| {
                ui.columns(3, |uis| {
                    uis[0].add(Gauge::horsepower(*horsepower).diameter(uis[0].available_width()));
                    uis[1].add(Gauge::rpm_roda(*rpm_roda).diameter(uis[1].available_width()));
                    uis[2].add(Gauge::torque(*torque).diameter(uis[2].available_width()));
                });
            });
        ui.vertical_centered(|ui| {
            Grid::new("dyno_left_values_grid_id")
                .num_columns(4)
                .spacing([ui.available_width() / 9.5, 4.0])
                .striped(true)
                .show(ui, grid_ui);
        });
    }

    pub fn right_panel(&mut self, ui: &mut Ui) {
        let Data {
            speed,
            rpm_engine,
            odo,
            ..
        } = self.buffer.last();
        const MULTPL_WIDTH: f32 = 0.19;
        const HEADING_SEGMENTS: [&str; 4] =
            ["Speed (km/h)", "RPM x 1000", "ODO (km)", "Time (HH:MM:SS)"];
        let value_segments = [
            format!("{:7.2}", speed.value()),
            format!("{:7.2}", rpm_engine.value() * 0.001),
            format!("{:7.2}", odo.value()),
            self.start_time(),
        ];
        let iter_segmented_ui = |(idx, segment_ui): (usize, &mut Ui)| {
            segment_ui.group(|uigroup_inner| {
                uigroup_inner.vertical_centered(|uivert_inner| {
                    uivert_inner.strong(HEADING_SEGMENTS[idx]);
                    let digit_height = uivert_inner.available_width() * MULTPL_WIDTH;
                    SegmentedDisplay::dyno_seven_segment(&value_segments[idx])
                        .style_preset(self.app_config.segment_display_style)
                        .digit_height(digit_height)
                        .ui(uivert_inner);
                });
            });
        };
        ui.columns(4, |segments_ui| {
            segments_ui
                .iter_mut()
                .enumerate()
                .for_each(iter_segmented_ui);
        });
        ui.separator();
        self.plots.ui(ui, &self.buffer);
    }

    #[inline]
    pub fn handle_states(&mut self, ctx: &Context) {
        if self.loadings.load(Ordering::Relaxed) {
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
    }
}

impl AsRef<DynoControl> for DynoControl {
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}
