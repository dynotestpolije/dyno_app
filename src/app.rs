use crate::{
    control::DynoControl,
    state::{DynoState, OperatorData},
    widgets::{gauges::Gauges, segment_display, toast::Toasts},
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

    #[cfg_attr(debug_assertions, serde(skip))]
    debug: crate::windows::DebugAction,
}

impl Default for Applications {
    fn default() -> Self {
        let control = DynoControl::default();
        Self {
            window_states: window_states_new(&control),
            control: DynoControl::new(),
            toast: Default::default(),
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
    #[inline]
    fn main_panels_draw(&mut self, ctx: &Context, data: &Data) {
        let uigroup_column_left = |left_ui: &mut Ui| {
            let width = left_ui.available_width() / 2.1;
            left_ui.horizontal(|uihorz| {
                uihorz.add(Gauges::speed(data.speed).diameter(width));
                uihorz.add(Gauges::rpm(data.rpm).diameter(width));
            });
            left_ui.separator();
            left_ui.horizontal(|uihorz| {
                uihorz.add(Gauges::horsepower(data.horsepower).diameter(width));
                uihorz.add(Gauges::torque(data.torque).diameter(width));
            });
        };
        let uigroup_column_right = |right_ui: &mut Ui| {
            right_ui.columns(2, |segments_ui| {
                segments_ui[0].group(|uigroup_inner| {
                    let digit_height = uigroup_inner.available_width() * 0.15;
                    uigroup_inner.heading("ODO (Km)");
                    uigroup_inner.add({
                        let mut segment = segment_display::SegmentedDisplay::dyno_seven_segment(
                            format!("{:07.2}", data.odo.to_float()),
                        )
                        .digit_height(digit_height);
                        if cfg!(debug_assertions) {
                            segment = segment.style_preset(self.debug.get_preset());
                        }
                        segment
                    });
                });
                segments_ui[1].group(|uigroup_inner| {
                    let digit_height = uigroup_inner.available_width() * 0.15;
                    uigroup_inner.heading("SPEED (Km/H)");
                    uigroup_inner.add({
                        let mut segment = segment_display::SegmentedDisplay::dyno_seven_segment(
                            format!("{:06.2}", data.speed.to_float()),
                        )
                        .digit_height(digit_height);
                        if cfg!(debug_assertions) {
                            segment = segment.style_preset(self.debug.get_preset());
                        }
                        segment
                    });
                });
            });
            right_ui.separator();
            self.control.show_plot(right_ui);
        };

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
            |uibottom| {
                uibottom.horizontal(|vertui| {
                    self.control.show_status(vertui);
                });
            },
        );

        SidePanel::left(PanelId::Left)
            .min_width(width * 0.3)
            .max_width(width * 0.5)
            .show_animated(ctx, self.state.show_left_panel(), uigroup_column_left);
        CentralPanel::default().show(ctx, uigroup_column_right);
    }
    #[inline]
    fn on_conditions(&mut self) {
        match self.state.get_operator() {
            OperatorData::SaveFile(tp) => match self.control.on_save(tp) {
                Ok(Some(path)) => {
                    self.toast
                        .info(format!("Saved {} file: {}", tp.as_str(), path.display()));
                }
                Err(err) => {
                    self.toast
                        .error(format!("Error on Saving {}: {err}", tp.as_str()));
                }
                _ => {}
            },
            OperatorData::OpenFile(tp) => {
                self.state.set_show_buffer_unsaved(true);
                match self.control.on_open(tp) {
                    Ok(p) if p.is_some() => self.toast.info(format!(
                        "Opened {} file: {}",
                        tp.as_str(),
                        p.unwrap().display()
                    )),
                    _ => self.toast.error("Failed Opening File"),
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for Applications {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let last_data = self.control.last_buffer();

        if cfg!(debug_assertions) {
            self.debug.show_window(ctx, self.control.buffer_mut());
        }
        for window in &mut self.window_states {
            window.show_window(ctx, frame, &mut self.state)
        }

        self.main_panels_draw(ctx, &last_data);
        self.on_conditions();

        self.toast.show(ctx);
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        visuals.window_fill().to_normalized_gamma_f32()
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {
        self.control.on_pos_render();
    }

    fn on_close_event(&mut self) -> bool {
        self.state.quitable() && (!self.state.show_buffer_unsaved())
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        // every 15 minute save ( 900 sec == 15 min )
        std::time::Duration::from_secs(900)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }
}
