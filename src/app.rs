use crate::{
    control::DynoControl,
    state::{DynoState, OperatorData},
    widgets::{button::ButtonExt, gauges::Gauges, segment_display, toast::Toasts},
    windows::{window_states_new, WindowState},
    PanelId, APP_KEY,
};
use dyno_types::{data_buffer::Data, Numeric};
use eframe::egui::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Applications {
    control: DynoControl,

    #[serde(skip)]
    window_states: Vec<Box<dyn WindowState>>,

    #[serde(skip)]
    toast: Toasts,

    state: DynoState,

    #[cfg(debug_assertions)]
    #[cfg_attr(debug_assertions, serde(skip))]
    debug: crate::windows::DebugAction,
}

impl Default for Applications {
    fn default() -> Self {
        let control = DynoControl::default();
        Self {
            window_states: window_states_new(&control),
            control: DynoControl::new(),
            toast: Toasts::new(),
            state: DynoState::new(),

            #[cfg(debug_assertions)]
            debug: Default::default(),
        }
    }
}

impl Applications {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        control: DynoControl,
    ) -> Box<dyn eframe::App + 'static> {
        Box::new(
            cc.storage
                .and_then(|s| eframe::get_value::<Self>(s, APP_KEY))
                .unwrap_or_else(|| Self {
                    window_states: window_states_new(&control),
                    control,
                    ..Default::default()
                }),
        )
    }
}

impl Applications {
    fn main_panels_draw(&mut self, ctx: &Context, data: &Data) {
        let width = ctx.available_rect().width();

        TopBottomPanel::top(PanelId::Top).show(ctx, |ui| {
            menu::bar(ui, |uibar| {
                widgets::global_dark_light_mode_switch(uibar);
                uibar.separator();
                self.state.menubar(uibar)
            })
        });

        TopBottomPanel::bottom(PanelId::Bottom).show_animated(
            ctx,
            self.state.show_bottom_panel(),
            |uibottom| self.bottom_status(uibottom),
        );

        let uigroup_column_left = |left_ui: &mut Ui| {
            left_ui.columns(2, |uis| {
                uis[0].add(Gauges::speed(data.speed).diameter(uis[0].available_width()));
                uis[1].add(Gauges::rpm(data.rpm).diameter(uis[1].available_width()));
            });
            left_ui.separator();
            left_ui.columns(3, |uis| {
                uis[0].add(Gauges::horsepower(data.horsepower).diameter(uis[0].available_width()));
                uis[1].add(
                    Gauges::horsepower(data.horsepower).diameter(uis[1].available_width() * 0.8),
                );
                uis[2].add(Gauges::torque(data.torque).diameter(uis[2].available_width()));
            });
        };
        const MULTPL_WIDTH: f32 = 0.19;
        const HEADING_SEGMENTS: [&str; 3] = ["ODO (km)", "Speed (km/h)", "Time (HH:MM:SS)"];
        let uigroup_column_right = |right_ui: &mut Ui| {
            right_ui.columns(3, |segments_ui| {
                let value_segments = [
                    format!("{:7.2}", data.odo.to_float()),
                    format!("{:7.2}", data.speed.to_float()),
                    self.control.start_time(data),
                ];
                segments_ui
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, segment_ui)| {
                        segment_ui.group(|uigroup_inner| {
                            uigroup_inner.vertical_centered(|uivert_inner| {
                                uivert_inner.strong(HEADING_SEGMENTS[idx]);
                                let digit_height = uivert_inner.available_width() * MULTPL_WIDTH;
                                uivert_inner.add({
                                    #[cfg(not(debug_assertions))]
                                    {
                                        segment_display::SegmentedDisplay::dyno_seven_segment(
                                            &value_segments[idx],
                                        )
                                        .digit_height(digit_height)
                                    }
                                    #[cfg(debug_assertions)]
                                    {
                                        segment_display::SegmentedDisplay::dyno_seven_segment(
                                            &value_segments[idx],
                                        )
                                        .style_preset(self.debug.get_preset())
                                        .digit_height(digit_height)
                                    }
                                });
                            });
                        });
                    });
            });
            right_ui.separator();
            crate::widgets::MultiRealtimePlot::new()
                .animate(true)
                .ui(right_ui, self.control.buffer());
        };

        SidePanel::left(PanelId::Left)
            .min_width(width * 0.3)
            .max_width(width * 0.5)
            .show_animated(ctx, self.state.show_left_panel(), uigroup_column_left);
        CentralPanel::default().show(ctx, uigroup_column_right);
    }

    fn bottom_status(&mut self, ui: &mut Ui) {
        ui.horizontal_centered(|vertui| {
            vertui.with_layout(Layout::left_to_right(Align::Center), |ltr_ui| {
                match self.control.service() {
                    Some(serial) => {
                        let crate::service::PortInfo {
                            port_name,
                            vid,
                            pid,
                            ..
                        } = serial.get_info();
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
                            if let Err(err) = self.control.reinitialize_service() {
                                self.toast.error(err);
                            } else if let Some(ser) = self.control.service() {
                                let info = ser.get_info();
                                self.toast.success(format!("SUCCES! connected to [{}] - [{}:{}]", info.port_name, info.vid, info.pid));
                            }
                        }
                    }
                }
                ltr_ui.separator();
                if ltr_ui
                    .small_play_button()
                    .on_hover_text("Click to Start the Service")
                    .clicked()
                {
                    if let Err(err) = self.control.service_start() {
                        self.toast.error(err);
                    }
                }
                if ltr_ui
                    .small_stop_button()
                    .on_hover_text("Click to Stop/Pause the Service")
                    .clicked()
                {
                    if let Some(serial) = self.control.service_mut() {
                        if let Err(err) = serial.send(crate::service::CmdMsg::Stop) {
                            self.toast.error(err);
                        }
                        serial.stop();
                    } else {
                        self.toast.error("[ERROR] Serial Port is not Initialize or not Connected!,\
                            try to Click on bottom left on 'STATUS' to reinitialize or reconnected");
                    }
                }
                if ltr_ui
                    .small_reset_button()
                    .on_hover_text("Click to Reset recorded data buffer")
                    .clicked()
                {
                    if let Some(ref mut serial) = self.control.service_mut() {
                        if let Err(err) = serial.send(crate::service::CmdMsg::Stop) {
                            self.toast.error(err);
                        }
                        serial.stop();
                        self.control.buffer_mut().clean();
                    } else {
                        self.toast.error("[ERROR] Serial Port is not Initialize or not Connected!,\
                            try to Click on bottom left on 'STATUS' to reinitialize or reconnected");
                    }
                }
            });
            vertui.separator();
            vertui.with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
                if let Ok(ref info) = self.control.info_motor().read() {
                    rtl_ui.small(format!("Active Info: {}", info));
                }
            });
        });
    }
}

impl eframe::App for Applications {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let last_data = self.control.last_buffer();

        #[cfg(debug_assertions)]
        {
            self.debug.show_window(ctx, self.control.buffer_mut());
        }
        for window in &mut self.window_states {
            window.show_window(ctx, &mut self.state)
        }

        match (self.state.get_operator(), self.control.is_buffer_saved()) {
            // if buffer is saved and operator want to save, do save the buffer, or if buffer
            // already saved, ignore the operator
            (OperatorData::SaveFile(tp), false) => {
                match self.control.on_save(tp) {
                    Ok(Some(path)) => self
                        .toast
                        .info(format!("Saved {tp} file: {}", path.display())),
                    Err(err) => self.toast.error(format!("Error on Saving {tp}: {err}")),
                    _ => {}
                };
                if self.state.quitable() {
                    self.state.set_quit(true);
                }
            }
            // if buffer is saved and operator want to open file, do open the file to buffer,
            // or is buffer unsaved but operator want ot open file, show popup to save buffer first
            (OperatorData::OpenFile(tp), true) => match self.control.on_open(tp) {
                Ok(Some(p)) => self
                    .toast
                    .info(format!("Opened {tp} file: {}", p.display())),
                Err(err) => self.toast.error(format!("Failed Opening File: {err}")),
                _ => {}
            },
            (OperatorData::OpenFile(_), false) => self.state.set_show_buffer_unsaved(true),
            _ => {}
        }

        self.toast.show(ctx);

        self.main_panels_draw(ctx, &last_data);

        if self.state.quit() {
            frame.close();
        }
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        self.control.on_pos_render();
    }

    fn on_close_event(&mut self) -> bool {
        if !self.control.is_buffer_saved() && !self.state.quit() {
            self.state.set_show_buffer_unsaved(true);
            return false;
        }
        if self.state.quitable() {
            true
        } else {
            self.state.set_show_quitable(true);
            false
        }
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // every 15 minute save ( 900 sec == 15 min )
        std::time::Duration::from_secs(900)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }
}
