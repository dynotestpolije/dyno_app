use crate::{
    control::DynoControl,
    state::DynoState,
    widgets::{gauges::Gauges, segment_display},
    windows::{window_states_new, WindowStack},
    PanelId, APP_KEY,
};
use dyno_core::{log, serde, Data, Numeric};
use eframe::egui::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
#[serde(default)]
pub struct Applications {
    control: DynoControl,

    #[serde(skip)]
    window_states: WindowStack,

    state: DynoState,

    #[cfg(debug_assertions)]
    #[cfg_attr(debug_assertions, serde(skip))]
    debug: crate::windows::DebugAction,
}

impl Default for Applications {
    fn default() -> Self {
        Self {
            window_states: window_states_new(),
            control: DynoControl::new(),
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
                    window_states: window_states_new(),
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
                self.state.menubar(uibar);
                uibar.with_layout(Layout::right_to_left(Align::Center), |rtl_ui| {
                    if rtl_ui.button("Login").clicked() {
                        log::info!("About submenu clicked");
                        self.state.swap_show_auth_window();
                    }
                });
            })
        });

        TopBottomPanel::bottom(PanelId::Bottom).show_animated(
            ctx,
            self.state.show_bottom_panel(),
            |uibottom| uibottom.horizontal_centered(|vertui| self.control.bottom_status(vertui)),
        );

        let uigroup_column_left = |left_ui: &mut Ui| {
            left_ui.columns(2, |uis| {
                uis[0].add(Gauges::speed(data.speed).diameter(uis[0].available_width()));
                uis[1].add(Gauges::rpm_engine(data.rpm_engine).diameter(uis[1].available_width()));
            });
            left_ui.separator();
            left_ui.columns(3, |uis| {
                uis[0].add(Gauges::horsepower(data.horsepower).diameter(uis[0].available_width()));
                uis[1]
                    .add(Gauges::rpm_roda(data.rpm_roda).diameter(uis[1].available_width() * 0.8));
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
                let iter_segmented_ui = |(idx, segment_ui): (usize, &mut Ui)| {
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
                };
                segments_ui
                    .iter_mut()
                    .enumerate()
                    .for_each(iter_segmented_ui);
            });
            right_ui.separator();
            self.control.show_plot(right_ui);
        };

        SidePanel::left(PanelId::Left)
            .min_width(width * 0.3)
            .max_width(width * 0.5)
            .show_animated(ctx, self.state.show_left_panel(), uigroup_column_left);
        CentralPanel::default().show(ctx, uigroup_column_right);
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
            window.show_window(ctx, &mut self.control, &mut self.state)
        }
        self.control.handle_states(ctx, frame, &mut self.state);

        crate::TOAST_MSG.lock().show(ctx);
        self.main_panels_draw(ctx, &last_data);
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
