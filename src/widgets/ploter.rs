use eframe::egui::plot::{LineStyle, LinkedAxisGroup, LinkedCursorsGroup, PlotPoints};
use eframe::egui::{plot, ComboBox, Response, TextStyle, Ui};
use std::ops::RangeInclusive;

// pub struct RealtimePlot {
// }

pub struct MultiRealtimePlot {
    line_style: LineStyle,
    group: LinkedAxisGroup,
    cursor_group: LinkedCursorsGroup,
    coordinates: bool,
    animates: bool,
}

impl Default for MultiRealtimePlot {
    fn default() -> Self {
        Self {
            line_style: plot::LineStyle::Solid,
            group: plot::LinkedAxisGroup::new(true, false),
            cursor_group: plot::LinkedCursorsGroup::new(true, false),
            animates: false,
            coordinates: true,
        }
    }
}

impl MultiRealtimePlot {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn animate(mut self, animeate: bool) -> Self {
        self.animates = animeate;
        self
    }

    pub fn options_ui(&mut self, ui: &mut Ui) {
        let Self {
            ref mut line_style,
            ref mut coordinates,
            ..
        } = self;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.style_mut().wrap = Some(false);
                ui.checkbox(coordinates, "Show coordinates")
                    .on_hover_text("Can take a custom formatting function.");

                ComboBox::from_label("Line style")
                    .selected_text(line_style.to_string())
                    .show_ui(ui, |ui| {
                        for style in &[
                            plot::LineStyle::Solid,
                            plot::LineStyle::dashed_dense(),
                            plot::LineStyle::dashed_loose(),
                            plot::LineStyle::dotted_dense(),
                            plot::LineStyle::dotted_loose(),
                        ] {
                            ui.selectable_value(line_style, *style, style.to_string());
                        }
                    });
            });
        });
    }

    #[inline(always)]
    fn x_axis_fmt(x: f64, _: &std::ops::RangeInclusive<f64>) -> String {
        format!("{:02}:{:02}", x as usize / 60, x as usize % 60)
    }

    const fn style_byidx(idx: usize) -> plot::LineStyle {
        match idx {
            0 => plot::LineStyle::Solid,
            1 => plot::LineStyle::Solid,
            2 => plot::LineStyle::Solid,
            3 => plot::LineStyle::Dashed { length: 5.0 },
            4 => plot::LineStyle::Dotted { spacing: 10.0 },
            _ => panic!("Unreachable index, index should be 0..=4"),
        }
    }
}

use dyno_types::data_buffer::BufferData;

impl MultiRealtimePlot {
    const LEGENDS: plot::Legend = plot::Legend {
        text_style: TextStyle::Monospace,
        background_alpha: 0.75,
        position: plot::Corner::RightTop,
    };

    pub fn ui(&mut self, ui: &mut Ui, data: &'_ BufferData) -> Response {
        if self.animates {
            ui.ctx().request_repaint();
        }
        let show_line_callback = |pui: &mut plot::PlotUi, range: RangeInclusive<usize>| {
            range
                .map(|index| {
                    plot::Line::new(data[index].into_points::<PlotPoints>())
                        .style(Self::style_byidx(index))
                        .name(BufferData::BUFFER_NAME[index])
                })
                .for_each(|line| pui.line(line))
        };
        ui.vertical_centered_justified(|ui| {
            let height = ui.available_height() * 0.5 - (ui.spacing().item_spacing.y * 2.0);
            let first = self.first_plot_io(ui, height, show_line_callback);
            ui.separator();
            let second = self.second_plot_ui(ui, height, show_line_callback);
            first.union(second)
        })
        .response
    }

    fn first_plot_io<F>(&mut self, ui: &mut Ui, height: f32, callback: F) -> Response
    where
        F: Fn(&mut plot::PlotUi, RangeInclusive<usize>),
    {
        plot::Plot::new("Speed and RPM Graph")
            .legend(Self::LEGENDS)
            .height(height)
            .x_axis_formatter(Self::x_axis_fmt)
            .link_axis(self.group.clone())
            .link_cursor(self.cursor_group.clone())
            .view_aspect(2.0)
            .coordinates_formatter(
                plot::Corner::LeftBottom,
                plot::CoordinatesFormatter::with_decimals(2),
            )
            .show(ui, |pui| callback(pui, 0..=1))
            .response
    }

    fn second_plot_ui<F>(&mut self, ui: &mut Ui, height: f32, callback: F) -> Response
    where
        F: Fn(&mut plot::PlotUi, RangeInclusive<usize>),
    {
        plot::Plot::new("Torque, Power and Temp Graph")
            .legend(Self::LEGENDS)
            .x_axis_formatter(Self::x_axis_fmt)
            .height(height)
            .link_axis(self.group.clone())
            .link_cursor(self.cursor_group.clone())
            .view_aspect(2.0)
            .coordinates_formatter(
                plot::Corner::LeftBottom,
                plot::CoordinatesFormatter::with_decimals(2),
            )
            .show(ui, |pui| callback(pui, 2..=4))
            .response
    }
}
