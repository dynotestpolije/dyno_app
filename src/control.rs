use crate::{
    config::ApplicationConfig,
    paths::DynoPaths,
    row_label_value,
    service::ServiceControl,
    state::{DynoFileType, DynoState, OperatorData},
    toast_error, toast_info, toast_success,
    widgets::{
        button::ButtonExt, segment_display::SegmentedDisplay, DynoFileManager, Gauge, RealtimePlot,
    },
    windows::{open_server::OpenServerWindow, WSIdx, WindowStack},
    AsyncMsg,
};
use dyno_core::{
    asyncify,
    chrono::{NaiveDateTime, Utc},
    ignore_err, log, serde, BufferData, CompresedSaver, CsvSaver, Data, DynoConfig, ExcelSaver,
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
    #[serde(skip)]
    #[serde(default = "ServiceControl::new")]
    pub service: ServiceControl,

    pub paths: DynoPaths,
    pub app_config: ApplicationConfig,
    pub config: DynoConfig,

    #[serde(skip)]
    #[serde(default)]
    buffer: BufferData,

    plots: RealtimePlot,

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
        Self {
            service: ServiceControl::new(),
            paths: Default::default(),
            app_config: Default::default(),
            config: Default::default(),
            buffer: Default::default(),
            plots: Default::default(),
            start_time: Default::default(),
            start: Default::default(),
            stop: Default::default(),
            loadings: Default::default(),
            buffer_saved: Default::default(),
        }
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
            ..Default::default()
        }
    }

    pub fn init(&mut self) {
        self.service.init(&self.config)
    }

    pub fn deinit(&mut self) {
        self.service.deinit();
    }

    #[inline(always)]
    pub fn last_buffer(&self) -> Data {
        *self.buffer.last()
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
    pub fn set_loading(&self) {
        self.loadings.store(true, Ordering::Relaxed);
    }
    #[inline]
    pub fn unset_loading(&self) {
        self.loadings.store(false, Ordering::Relaxed);
    }
}

impl DynoControl {
    #[inline]
    pub fn on_pos_render(&mut self, window_stack: &mut WindowStack, state: &mut DynoState) {
        match self.service.msg() {
            AsyncMsg::OnSerialData(serial_data) => {
                self.start_time += serial_data.period as u64;
                self.buffer.push_from_serial(&mut self.config, serial_data);
                self.buffer_saved = false;
            }
            AsyncMsg::OnOpenBuffer(buffer) => {
                self.buffer = *buffer;
                self.buffer_saved = false;
                self.unset_loading();
            }
            AsyncMsg::OnError(err) => {
                toast_error!("{err}");
                self.unset_loading();
            }
            AsyncMsg::OnSavedBuffer(()) => {
                self.buffer_saved = true;
                if state.quitable() {
                    state.set_quit(true);
                }
                self.unset_loading();
            }
            AsyncMsg::OnCheckHealthApi(s) => {
                if s.is_success() {
                    toast_success!("API Check Health is Success");
                }
                self.unset_loading();
            }
            AsyncMsg::OnMessage(msg) => toast_info!("{msg}"),
            AsyncMsg::OnApiLoadDyno(data) => {
                match window_stack.idx_mut::<OpenServerWindow>(WSIdx::OpenServer) {
                    Some(window) => window.set_data(data),
                    None => dyno_core::log::error!("Failed to Downcast winddow stack"),
                }
                self.unset_loading();
            }
            AsyncMsg::OnApiLogin | AsyncMsg::OnApiRegister => {
                window_stack.set_swap_open(WSIdx::Auth);
                self.unset_loading();
            }
            AsyncMsg::Noop => {}
        }

        match (state.get_operator(), self.is_buffer_saved()) {
            // if buffer is saved and operator want to save, do save the buffer, or if buffer
            // already saved, ignore the operator
            (OperatorData::SaveFile(tp), false) => self.on_save(tp),
            // if buffer is saved and operator want to open file, do open the file to buffer,
            // or is buffer unsaved but operator want ot open file, show popup to save buffer first
            (OperatorData::OpenFile(tp), true) => self.on_open(tp),
            (OperatorData::OpenFile(_), false) => {
                window_stack.set_open(WSIdx::ConfirmUnsaved, true)
            }
            _ => {}
        }
    }
    pub fn on_save(&mut self, tp: DynoFileType) {
        use dyno_core::tokio;

        let buffer = self.buffer.clone();
        let loadings = self.loadings.clone();
        let tx = self.service.tx();

        let dirpath = tp.path(self.paths.get_data_dir_folder("Saved"));
        tokio::spawn(async move {
            loadings.store(true, Ordering::Relaxed);
            match tp {
                DynoFileType::Dyno => {
                    match DynoFileManager::save_binaries_async(
                        format!("dynotest_{}.dyno", Utc::now().timestamp()),
                        dirpath,
                    )
                    .await
                    {
                        Some(file) => match asyncify!(move || buffer.compress_to_path(file.path()))
                        {
                            Ok(()) => ignore_err!(tx.send(AsyncMsg::OnSavedBuffer(()))),
                            Err(err) => ignore_err!(tx.send(AsyncMsg::OnError(err))),
                        },
                        None => dyno_core::log::debug!("FileManager ppick file canceled"),
                    }
                }
                DynoFileType::Csv => {
                    match DynoFileManager::save_csv_async(
                        format!("dynotest_{}.csv", Utc::now().timestamp()),
                        dirpath,
                    )
                    .await
                    {
                        Some(file) => {
                            match asyncify!(move || buffer.save_csv_from_path(file.path())) {
                                Ok(()) => ignore_err!(tx.send(AsyncMsg::OnSavedBuffer(()))),
                                Err(err) => ignore_err!(tx.send(AsyncMsg::OnError(err))),
                            }
                        }
                        None => dyno_core::log::debug!("FileManager ppick file canceled"),
                    }
                }
                DynoFileType::Excel => match DynoFileManager::save_excel_async(
                    format!("dynotest_{}.xlsx", Utc::now().timestamp()),
                    dirpath,
                )
                .await
                {
                    Some(file) => match asyncify!(move || buffer.save_excel_from_path(file.path()))
                    {
                        Ok(()) => ignore_err!(tx.send(AsyncMsg::OnSavedBuffer(()))),
                        Err(err) => ignore_err!(tx.send(AsyncMsg::OnError(err))),
                    },
                    None => dyno_core::log::debug!("FileManager ppick file canceled"),
                },
            };
            loadings.store(false, Ordering::Relaxed);
        });
    }

    pub fn on_open(&mut self, tp: DynoFileType) {
        use dyno_core::tokio;

        let tx = self.service.tx();
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
    pub fn top_panel(
        &mut self,
        ui: &mut Ui,
        window_stack: &mut WindowStack,
        state: &mut DynoState,
    ) {
        ui.menu_button("File", |menu_ui| {
            if menu_ui.open_button().clicked() {
                log::debug!("Open Button menu clicked");
                state.set_operator(OperatorData::OpenFile(DynoFileType::Dyno));
            }
            menu_ui.menu_button("Open As..", |submenu_ui| {
                if submenu_ui.button("Csv File").clicked() {
                    state.set_operator(OperatorData::OpenFile(DynoFileType::Csv));
                    log::debug!("Open as Csv file submenu clicked");
                }
                if submenu_ui.button("Excel File").clicked() {
                    state.set_operator(OperatorData::OpenFile(DynoFileType::Excel));
                    log::debug!("Open as Excel file submenu clicked");
                }
                if submenu_ui.button("Binaries File").clicked() {
                    state.set_operator(OperatorData::OpenFile(DynoFileType::Dyno));
                    log::debug!("Open as Binaries file submenu clicked");
                }
            });
            if menu_ui.save_button().clicked() {
                log::debug!("Save file menu clicked");
                state.set_operator(OperatorData::SaveFile(DynoFileType::Dyno));
            }
            menu_ui.menu_button("Save As..", |submenu_ui| {
                if submenu_ui.button("Csv File").clicked() {
                    log::debug!("Save as Csv file submenu clicked");
                    state.set_operator(OperatorData::SaveFile(DynoFileType::Csv));
                }
                if submenu_ui.button("Excel File").clicked() {
                    log::debug!("Save as Excel file submenu clicked");
                    state.set_operator(OperatorData::SaveFile(DynoFileType::Excel));
                }
                if submenu_ui.button("Binaries File").clicked() {
                    log::debug!("Save as Binaries file submenu clicked");
                    state.set_operator(OperatorData::SaveFile(DynoFileType::Dyno));
                }
            });
            if menu_ui.button("Quit").clicked() {
                log::debug!("Exit submenu clicked");
                window_stack.set_open(WSIdx::ConfirmQuit, true);
            }
        });
        ui.menu_button("View", |submenu_ui| {
            submenu_ui.checkbox(state.show_bottom_panel_mut(), "Bottom Panel");
            submenu_ui.checkbox(state.show_left_panel_mut(), "Left Panel");
            if submenu_ui
                .checkbox(state.show_logger_window_mut(), "Logger Window")
                .changed()
            {
                window_stack.set_open(WSIdx::Logger, state.show_logger_window())
            }
        });
        if ui.button("Config").clicked() {
            log::debug!("Config submenu clicked");
            window_stack.set_swap_open(WSIdx::Setting);
        }
        if ui.button("Help").clicked() {
            log::debug!("Help submenu clicked");
            window_stack.set_swap_open(WSIdx::Help);
        }
        if ui.button("About").clicked() {
            log::debug!("About submenu clicked");
            window_stack.set_swap_open(WSIdx::About);
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
            eframe::egui::widgets::global_dark_light_mode_switch(rtl_ui);
            match self.service.api() {
                Some(api) if api.is_logined() => {
                    if rtl_ui.button("Logout").clicked() {
                        log::info!("Logout button clicked");
                        api.logout(self.service.tx());
                    }
                    if rtl_ui.button("Open from Server").clicked() {
                        log::info!("Opening Window Open Project from server...");
                        window_stack.set_swap_open(WSIdx::OpenServer);
                    }
                    if rtl_ui.button("Save to Server").clicked() {
                        log::info!("Opening Window Save Project to to server...");
                        window_stack.set_swap_open(WSIdx::SaveServer);
                    }
                }
                _ => {
                    if rtl_ui
                        .button("Login")
                        .on_hover_text("login first to access server, like saving data to server.")
                        .clicked()
                    {
                        log::info!("Login bottom clicked");
                        window_stack.set_swap_open(WSIdx::Auth);
                    }
                }
            }
        });
    }

    #[inline(always)]
    pub fn bottom_status(&mut self, ui: &mut Ui) {
        let layout_ui_status = |ltr_ui: &mut Ui| match self.service.serial() {
            Some(serial) => {
                let serial_open = serial.is_open();
                let (status, color) = if serial_open {
                    ("STATUS: Running", Color32::BLUE)
                } else {
                    ("STATUS: Connected", Color32::GREEN)
                };
                let info = serial.get_info();
                Label::new(RichText::new(status).color(color))
                    .ui(ltr_ui)
                    .on_hover_text(format!(
                        "PORT INFO: [{}] ({}:{})",
                        info.port_name, info.vid, info.pid
                    ));
                ltr_ui.separator();
                let btn_start = ltr_ui
                    .small_play_button()
                    .on_hover_text("Click to Start the Service");
                let btn_stop = ltr_ui
                    .small_stop_button()
                    .on_hover_text("Click to Stop/Pause the Service");
                let btn_reset = ltr_ui
                    .small_reset_button()
                    .on_hover_text("Click to Stop and Reset recorded data buffer");
                match (
                    btn_start.clicked(),
                    btn_stop.clicked(),
                    btn_reset.clicked(),
                    serial_open,
                ) {
                    (true, _, _, false) => {
                        if let Err(err) = serial.start(self.service.tx()) {
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
                    (_, _, true, _) => self.buffer.clean(),
                    _ => {}
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
                    self.service.reconnect_serial();
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
