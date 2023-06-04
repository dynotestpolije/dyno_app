use dyno_core::{serde, AsStr, BufferData, PointShowed};
use eframe::egui::*;
use std::hash::Hash;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum PlotPanel {
    #[default]
    All,
    First,
    Second,
}
impl AsStr<'static> for PlotPanel {
    fn as_str(&self) -> &'static str {
        match self {
            PlotPanel::All => "All Plot",
            PlotPanel::First => "First Plot",
            PlotPanel::Second => "Second Plot",
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
pub struct RealtimePlot {
    group: Id,
    cursor_group: Id,

    first_showed_x: i64,
    first_x: i64,
    last_x: i64,

    panel: PlotPanel,
    showed: PointShowed,

    allow_drag: bool,
    allow_zoom: bool,
    allow_scroll: bool,
    allow_boxed_zoom: bool,
    animates: bool,
}

impl Default for RealtimePlot {
    fn default() -> Self {
        let group = Id::new("dyno_plot_group_id");
        let cursor_group = Id::new("dyno_plot_cursor_group_id");
        Self {
            group,
            cursor_group,
            panel: Default::default(),
            showed: PointShowed::default(),
            first_showed_x: 0,
            first_x: 0,
            last_x: 0,

            allow_drag: true,
            allow_zoom: true,
            allow_scroll: true,
            allow_boxed_zoom: true,
            animates: false,
        }
    }
}

impl RealtimePlot {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RealtimePlot {
    const LEGENDS: plot::Legend = plot::Legend {
        text_style: TextStyle::Monospace,
        background_alpha: 0.75,
        position: plot::Corner::RightTop,
    };

    pub fn ui(&mut self, ui: &mut Ui, data: &'_ BufferData) -> Response {
        if self.animates {
            ui.ctx().request_repaint();
        }
        ui.horizontal(|ui| {
            ui.collapsing("Instructions", |ui| {
                ui.label("Pan by dragging, or scroll (+ shift = horizontal).");
                ui.label(
                    "Box zooming: Middle (Scroll) click to zoom in and zoom out using a selection.",
                );
                ui.label("Zoom with ctrl + scroll.");
                ui.label("Reset view with double-click.");
                ui.label("Change behaviour with context menu right click in the plot");
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |left_ui| {
                ComboBox::new("plot_panel_combobox", "Select Plot to Show")
                    .selected_text(self.panel.as_str())
                    .show_ui(left_ui, |ui| {
                        ui.selectable_value(
                            &mut self.panel,
                            PlotPanel::All,
                            PlotPanel::All.as_str(),
                        );
                        ui.selectable_value(
                            &mut self.panel,
                            PlotPanel::First,
                            PlotPanel::First.as_str(),
                        );
                        ui.selectable_value(
                            &mut self.panel,
                            PlotPanel::Second,
                            PlotPanel::Second.as_str(),
                        );
                    });
                left_ui.menu_button("Plot Config", |cfg_ui| {
                    Grid::new("button_grid_plot").show(cfg_ui, |gridui| {
                        gridui.checkbox(&mut self.allow_drag, "Drag");
                        gridui.checkbox(&mut self.allow_zoom, "Zoom");
                        gridui.end_row();
                        gridui.checkbox(&mut self.allow_scroll, "Scroll");
                        gridui.checkbox(&mut self.allow_boxed_zoom, "Boxed Zoom");
                        gridui.end_row();
                        gridui.checkbox(&mut self.animates, "Animate");
                    });
                })
            });
        });
        let showed = self.showed;

        self.first_x = data.time_stamp.first_value();
        self.last_x = data.time_stamp.last_value();
        ui.horizontal(|ui| {
            ComboBox::new("point_showed_combobox", "Points to Show")
                .selected_text(format!("{}", self.showed))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.showed, PointShowed::All, "All Points");
                    ui.selectable_value(&mut self.showed, PointShowed::Half, "Half Points");
                    ui.selectable_value(&mut self.showed, PointShowed::Quarter, "Quarter Points");
                    let slider = ui.add(
                        Slider::new(&mut self.first_showed_x, 0..=(self.last_x - self.first_x))
                            .step_by(1000f64)
                            .text("Show From")
                            .custom_formatter(|x, _| timestamp_diff_fmt(x as _)),
                    );

                    if slider.changed() {
                        self.showed = PointShowed::Num(
                            data.time_stamp
                                .iter()
                                .position(|x| *x == self.first_showed_x)
                                .unwrap_or_default(),
                        )
                    }
                });
        });

        let draw_plot_first = |pui: &mut plot::PlotUi| {
            pui.line(
                plot::Line::new(data.speed.into_points::<plot::PlotPoints>(showed))
                    .width(3.0)
                    .style(plot::LineStyle::Solid)
                    .name("Speed (km/h)"),
            );
            pui.line(
                plot::Line::new(
                    data.rpm_engine
                        .into_points_map::<plot::PlotPoints, _>(showed, |x| x * 0.001),
                )
                .width(3.0)
                .style(plot::LineStyle::dashed_dense())
                .name("RPM (engine) (rpm x 1000)"),
            );
            pui.line(
                plot::Line::new(data.temp.into_points::<plot::PlotPoints>(showed))
                    .width(3.0)
                    .style(plot::LineStyle::dashed_loose())
                    .name("Temp (°C)"),
            );
        };
        let draw_plot_second = |pui: &mut plot::PlotUi| {
            pui.line(
                plot::Line::new(data.torque.into_points::<plot::PlotPoints>(showed))
                    .width(3.0)
                    .style(plot::LineStyle::Solid)
                    .name("Torque (Nm)"),
            );
            pui.line(
                plot::Line::new(data.horsepower.into_points::<plot::PlotPoints>(showed))
                    .width(3.0)
                    .style(plot::LineStyle::Solid)
                    .name("HorsePower (HP)"),
            );
            pui.line(
                plot::Line::new(
                    data.rpm_roda
                        .into_points_map::<plot::PlotPoints, _>(showed, |x| x * 0.001),
                )
                .width(3.0)
                .style(plot::LineStyle::dashed_dense())
                .name("RPM (roda) (rpm x 1000)"),
            );
        };

        ui.vertical_centered(|ui| {
            let width = ui.available_width() - (ui.spacing().item_spacing.x * 2.0);
            let height = ui.available_height() - (ui.spacing().item_spacing.y * 2.0);
            match self.panel {
                PlotPanel::All => {
                    let height = height * 0.5;
                    self.draw_plot(ui, "dyno_plot_first", height, width, draw_plot_first);
                    ui.separator();
                    self.draw_plot(ui, "dyno_plot_second", height, width, draw_plot_second)
                }
                PlotPanel::First => {
                    self.draw_plot(ui, "dyno_plot_first", height, width, draw_plot_first)
                }
                PlotPanel::Second => {
                    self.draw_plot(ui, "dyno_plot_second", height, width, draw_plot_second)
                }
            }
        })
        .response
    }

    #[inline]
    fn draw_plot<F, R, S>(
        &mut self,
        ui: &mut Ui,
        name: S,
        height: f32,
        width: f32,
        draw_call: F,
    ) -> Response
    where
        F: FnOnce(&mut plot::PlotUi) -> R,
        S: Hash,
    {
        plot::Plot::new(name)
            .legend(Self::LEGENDS)
            .height(height)
            .width(width)
            .x_axis_formatter(|x, _| timestamp_diff_fmt(x as _))
            .coordinates_formatter(
                plot::Corner::LeftBottom,
                plot::CoordinatesFormatter::with_decimals(2),
            )
            .link_cursor(self.cursor_group, true, true)
            .link_axis(self.group, true, true)
            .allow_drag(self.allow_drag)
            .allow_scroll(self.allow_scroll)
            .allow_zoom(self.allow_zoom)
            .allow_boxed_zoom(self.allow_boxed_zoom)
            .show(ui, draw_call)
            .response
    }
}

#[inline]
fn timestamp_diff_fmt(timestamp: i64) -> String {
    let seconds = timestamp / 1000;
    format!(
        "{h:02}:{m:02}:{s:02}:{ms:03}",
        h = seconds / 3600,
        m = (seconds % 3600) / 60,
        s = seconds % 60,
        ms = timestamp % 1000
    )
}
