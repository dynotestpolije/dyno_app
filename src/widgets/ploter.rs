use dyno_core::{serde, AsStr, BufferData, PointShowed};
use eframe::egui::*;
use std::hash::Hash;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "serde")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(crate = "serde")]
pub struct MultiRealtimePlot {
    group: Id,
    cursor_group: Id,

    last_x: f64,
    panel: PlotPanel,
    showed: PointShowed,

    allow_drag: bool,
    allow_zoom: bool,
    allow_scroll: bool,
    allow_boxed_zoom: bool,
    animates: bool,
}

impl Default for MultiRealtimePlot {
    fn default() -> Self {
        Self {
            panel: Default::default(),
            group: Id::new("dyno_plot_group_id"),
            cursor_group: Id::new("dyno_plot_cursor_group_id"),
            showed: PointShowed::default(),
            last_x: 0.0,

            allow_drag: true,
            allow_zoom: true,
            allow_scroll: true,
            allow_boxed_zoom: true,
            animates: false,
        }
    }
}

impl MultiRealtimePlot {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MultiRealtimePlot {
    // pub fn ui_config(arg: Type) -> RetType {
    //     unimplemented!();
    // }

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
                ComboBox::new("point_showed_combobox", "Points to Show")
                    .selected_text(format!("{}", self.showed))
                    .show_ui(left_ui, |ui| {
                        ui.selectable_value(&mut self.showed, PointShowed::All, "All Points");
                        ui.selectable_value(&mut self.showed, PointShowed::Half, "Half Points");
                        ui.selectable_value(
                            &mut self.showed,
                            PointShowed::Quarter,
                            "Quarter Points",
                        );
                    });
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

        let draw_plot_first = |pui: &mut plot::PlotUi| {
            pui.line(
                plot::Line::new(data.torque.into_points::<plot::PlotPoints>(showed))
                    .style(plot::LineStyle::Solid)
                    .name("Torque (Nm)"),
            );
            pui.line(
                plot::Line::new(data.horsepower.into_points::<plot::PlotPoints>(showed))
                    .style(plot::LineStyle::Solid)
                    .name("HorsePower (HP)"),
            );
            pui.line(
                plot::Line::new(
                    data.rpm_roda
                        .into_points_map::<plot::PlotPoints, _>(showed, |x| x * 0.001),
                )
                .style(plot::LineStyle::dashed_dense())
                .name("RPM (roda) (rpm x 1000)"),
            );
        };
        let draw_plot_second = |pui: &mut plot::PlotUi| {
            pui.line(
                plot::Line::new(data.speed.into_points::<plot::PlotPoints>(showed))
                    .style(plot::LineStyle::Solid)
                    .name("Speed (km/h)"),
            );
            pui.line(
                plot::Line::new(
                    data.rpm_engine
                        .into_points_map::<plot::PlotPoints, _>(showed, |x| x * 0.001),
                )
                .style(plot::LineStyle::dotted_dense())
                .name("RPM (engine) (rpm x 1000)"),
            );
            pui.line(
                plot::Line::new(data.temp.into_points::<plot::PlotPoints>(showed))
                    .style(plot::LineStyle::dotted_loose())
                    .name("Temp (°C)"),
            );
        };
        self.last_x = data.len() as _;

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
            .x_axis_formatter(|x, _r| {
                let x = x * 250.;
                format!("{:02}:{:02}", (x * 0.01666) as usize, x as usize % 60)
            })
            .link_cursor(self.cursor_group, true, true)
            .link_axis(self.group, true, true)
            .coordinates_formatter(
                plot::Corner::LeftBottom,
                plot::CoordinatesFormatter::with_decimals(2),
            )
            .allow_drag(self.allow_drag)
            .allow_scroll(self.allow_scroll)
            .allow_zoom(self.allow_zoom)
            .allow_boxed_zoom(self.allow_boxed_zoom)
            .show(ui, draw_call)
            .response
    }
}
