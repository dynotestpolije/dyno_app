pub mod button;
pub mod gauges;
pub mod hyperlink;
pub mod images;
pub mod logger;
pub mod msgdialog;
pub mod popup;
pub mod toast;

pub mod segment_display;

mod common;
mod opener;
mod ploter;

#[cfg(feature = "barcodes")]
pub mod barcodes;

pub use opener::{DynoFileManager, Filters};
pub use ploter::MultiRealtimePlot;

pub use common::*;

// ----------------------------------------------------------------------------
pub trait DynoWidgets: button::ButtonExt {
    fn seven_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>;

    fn nine_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>;

    fn sixteen_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>;

    fn hyperlink_with_icon<U>(&mut self, url: U) -> eframe::egui::Response
    where
        U: AsRef<str>;

    fn hyperlink_with_icon_to<S, U>(&mut self, label: S, url: U) -> eframe::egui::Response
    where
        S: AsRef<str>,
        U: AsRef<str>;

    fn drag_rangeinclusive<T: eframe::emath::Numeric>(
        &mut self,
        value: &mut std::ops::RangeInclusive<T>,
    ) -> eframe::egui::Response;

    fn optional_value_widget<T: Default>(
        &mut self,
        value: &mut Option<T>,
        add_contents: impl FnOnce(&mut Self, &mut T) -> eframe::egui::Response,
    ) -> eframe::egui::Response;

    fn selectable_value_from_iter<V>(
        &mut self,
        current_value: &mut V,
        values: impl Iterator<Item = V>,
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display;

    fn radio_value_from_iter<V>(
        &mut self,
        current_value: &mut V,
        values: impl Iterator<Item = V>,
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display;

    fn combobox_from_iter<V>(
        &mut self,
        label: impl Into<eframe::egui::WidgetText>,
        current_value: &mut V,
        values: impl Iterator<Item = V>,
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display;

    fn selectable_value_from_slice<V>(
        &mut self,
        current_value: &mut V,
        values: &'_ [V],
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display;

    fn radio_value_from_slice<V>(
        &mut self,
        current_value: &mut V,
        values: &'_ [V],
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display;

    fn combobox_from_slice<V>(
        &mut self,
        label: impl Into<eframe::egui::WidgetText>,
        current_value: &mut V,
        values: &'_ [V],
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display;

    fn selectable_label_from_slice<W>(
        &mut self,
        slices: &'_ mut [bool],
        label: impl FnMut(usize) -> W,
    ) -> eframe::egui::Response
    where
        W: Into<eframe::egui::WidgetText>;
}

impl DynoWidgets for eframe::egui::Ui {
    #[inline(always)]
    fn seven_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>,
    {
        use segment_display::SegmentedDisplay;
        self.add(SegmentedDisplay::seven_segment(value))
    }

    #[inline(always)]
    fn nine_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>,
    {
        use segment_display::SegmentedDisplay;
        self.add(SegmentedDisplay::seven_segment(value))
    }

    #[inline(always)]
    fn sixteen_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>,
    {
        use segment_display::SegmentedDisplay;
        self.add(SegmentedDisplay::seven_segment(value))
    }

    #[inline(always)]
    fn hyperlink_with_icon<U>(&mut self, url: U) -> eframe::egui::Response
    where
        U: AsRef<str>,
    {
        self.add(hyperlink::hyperlink_with_icon(url))
    }

    #[inline(always)]
    fn hyperlink_with_icon_to<S, U>(&mut self, label: S, url: U) -> eframe::egui::Response
    where
        S: AsRef<str>,
        U: AsRef<str>,
    {
        self.add(hyperlink::hyperlink_with_icon_to(label, url))
    }

    fn optional_value_widget<T: Default>(
        &mut self,
        value: &mut Option<T>,
        add_contents: impl FnOnce(&mut Self, &mut T) -> eframe::egui::Response,
    ) -> eframe::egui::Response {
        self.group(|ui| {
            ui.horizontal(|ui| {
                let mut checkbox_state = value.is_some();
                let mut response = ui.checkbox(&mut checkbox_state, "");
                match (value.is_some(), checkbox_state) {
                    (false, true) => *value = Some(T::default()),
                    (true, false) => *value = None,
                    _ => {}
                };
                match value {
                    Some(ref mut value) => {
                        response = response.union(add_contents(ui, value));
                    }
                    None => {
                        let mut dummy_value = T::default();
                        ui.add_enabled_ui(false, |ui| add_contents(ui, &mut dummy_value));
                    }
                }
                response
            })
        })
        .inner
        .inner
    }

    fn drag_rangeinclusive<T: eframe::emath::Numeric>(
        &mut self,
        value: &mut std::ops::RangeInclusive<T>,
    ) -> eframe::egui::Response {
        self.group(|ui| {
            let (mut start, mut end) = (*value.start(), *value.end());
            ui.add(eframe::egui::DragValue::new(&mut start));
            ui.add(eframe::egui::DragValue::new(&mut end));
            *value = start..=end;
        })
        .response
    }

    fn selectable_value_from_iter<V>(
        &mut self,
        current_value: &mut V,
        values: impl Iterator<Item = V>,
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display,
    {
        values
            .map(|value| self.selectable_value(current_value, value, format!("{value}")))
            .reduce(|result, response| result.union(response))
            .unwrap_or_else(|| {
                self.colored_label(self.style().visuals.error_fg_color, "\u{1F525} No items")
            })
    }

    fn radio_value_from_iter<V>(
        &mut self,
        current_value: &mut V,
        values: impl Iterator<Item = V>,
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display,
    {
        values
            .map(|value| self.radio_value(current_value, value, format!("{value}")))
            .reduce(|result, response| result.union(response))
            .unwrap_or_else(|| {
                self.colored_label(self.style().visuals.error_fg_color, "\u{1F525} No items")
            })
    }

    fn combobox_from_iter<V>(
        &mut self,
        label: impl Into<eframe::egui::WidgetText>,
        current_value: &mut V,
        values: impl Iterator<Item = V>,
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + std::fmt::Display,
    {
        let combobox_response = eframe::egui::ComboBox::from_label(label)
            .selected_text(format!("{current_value}"))
            .show_ui(self, |ui| {
                values
                    .map(|value| ui.selectable_value(current_value, value, format!("{value}")))
                    .reduce(|result, response| result.union(response))
                    .unwrap_or_else(|| {
                        ui.colored_label(ui.style().visuals.error_fg_color, "\u{1F525} No items")
                    })
            });
        combobox_response
            .inner
            .unwrap_or(combobox_response.response)
    }

    fn selectable_value_from_slice<V>(
        &mut self,
        current_value: &mut V,
        values: &'_ [V],
    ) -> eframe::egui::Response
    where
        V: Clone + PartialEq + std::fmt::Display,
    {
        values
            .iter()
            .map(|value| self.selectable_value(current_value, value.clone(), format!("{value}")))
            .reduce(|result, response| result.union(response))
            .unwrap_or_else(|| {
                self.colored_label(self.style().visuals.error_fg_color, "\u{1F525} No items")
            })
    }
    fn radio_value_from_slice<V>(
        &mut self,
        current_value: &mut V,
        values: &'_ [V],
    ) -> eframe::egui::Response
    where
        V: Clone + PartialEq + std::fmt::Display,
    {
        values
            .iter()
            .map(|value| self.radio_value(current_value, value.clone(), format!("{value}")))
            .reduce(|result, response| result.union(response))
            .unwrap_or_else(|| {
                self.colored_label(self.style().visuals.error_fg_color, "\u{1F525} No items")
            })
    }
    fn combobox_from_slice<V>(
        &mut self,
        label: impl Into<eframe::egui::WidgetText>,
        current_value: &mut V,
        values: &'_ [V],
    ) -> eframe::egui::Response
    where
        V: Clone + PartialEq + std::fmt::Display,
    {
        let combobox = eframe::egui::ComboBox::from_label(label)
            .selected_text(format!("{current_value}"))
            .show_ui(self, |ui| {
                values
                    .iter()
                    .map(|value| {
                        ui.selectable_value(current_value, value.clone(), format!("{value}"))
                    })
                    .reduce(|result, response| result.union(response))
                    .unwrap_or_else(|| {
                        ui.colored_label(ui.style().visuals.error_fg_color, "\u{1F525} No items")
                    })
            });
        combobox.inner.unwrap_or(combobox.response)
    }
    #[inline(always)]
    fn selectable_label_from_slice<W>(
        &mut self,
        slices: &'_ mut [bool],
        mut label: impl FnMut(usize) -> W,
    ) -> eframe::egui::Response
    where
        W: Into<eframe::egui::WidgetText>,
    {
        slices
            .iter_mut()
            .enumerate()
            .map(|(idx, item)| self.toggle_value(item, label(idx)))
            .reduce(|result, response| result.union(response))
            .unwrap_or_else(|| {
                self.colored_label(self.style().visuals.error_fg_color, "\u{1F525} No items")
            })
    }
}

pub trait PainterHelper: RotatedText {
    fn text_center_monospace(
        &self,
        pos: impl Into<eframe::emath::Pos2>,
        text: impl ToString,
        text_size: f32,
        color: impl Into<eframe::epaint::Color32>,
        wrap_width: Option<f32>,
    );
    fn text_center_prop(
        &self,
        pos: impl Into<eframe::emath::Pos2>,
        text: impl ToString,
        text_size: f32,
        color: impl Into<eframe::epaint::Color32>,
        wrap_width: Option<f32>,
    );
}

impl PainterHelper for eframe::egui::Painter {
    #[inline(always)]
    fn text_center_monospace(
        &self,
        pos: impl Into<eframe::emath::Pos2>,
        text: impl ToString,
        text_size: f32,
        color: impl Into<eframe::epaint::Color32>,
        wrap_width: Option<f32>,
    ) {
        let fontid = eframe::egui::FontId::monospace(text_size);
        let galley = if let Some(width) = wrap_width {
            self.layout(text.to_string(), fontid, color.into(), width)
        } else {
            self.layout_no_wrap(text.to_string(), fontid, color.into())
        };
        let rect = eframe::egui::Align2::CENTER_CENTER
            .anchor_rect(eframe::egui::Rect::from_min_size(pos.into(), galley.size()));

        if !galley.is_empty() {
            self.add(eframe::egui::Shape::galley(rect.min, galley));
        }
    }
    #[inline(always)]
    fn text_center_prop(
        &self,
        pos: impl Into<eframe::emath::Pos2>,
        text: impl ToString,
        text_size: f32,
        color: impl Into<eframe::epaint::Color32>,
        wrap_width: Option<f32>,
    ) {
        let fontid = eframe::egui::FontId::proportional(text_size);
        let galley = if let Some(width) = wrap_width {
            self.layout(text.to_string(), fontid, color.into(), width)
        } else {
            self.layout_no_wrap(text.to_string(), fontid, color.into())
        };
        let rect = eframe::egui::Align2::CENTER_CENTER
            .anchor_rect(eframe::egui::Rect::from_min_size(pos.into(), galley.size()));

        if !galley.is_empty() {
            self.add(eframe::egui::Shape::galley(rect.min, galley));
        }
    }
}

pub trait ImplResponseEgui {
    fn clicked_and(&self, callback: impl FnOnce());
    fn clicked_set<T>(&self, set: &mut T, val: T);
    fn clicked_swap<N: std::ops::Not<Output = N> + Copy>(&self, set: &mut N);
}

impl ImplResponseEgui for eframe::egui::Response {
    #[inline(always)]
    fn clicked_and(&self, callback: impl FnOnce()) {
        if self.clicked[eframe::egui::PointerButton::Primary as usize] {
            callback()
        }
    }

    #[inline(always)]
    fn clicked_set<T: Sized>(&self, set: &mut T, val: T) {
        if self.clicked[eframe::egui::PointerButton::Primary as usize] {
            *set = val
        }
    }

    #[inline(always)]
    fn clicked_swap<N: std::ops::Not<Output = N> + Copy>(&self, set: &mut N) {
        if self.clicked[eframe::egui::PointerButton::Primary as usize] {
            *set = std::ops::Not::not(*set);
        }
    }
}

// ----------------------------------------------------------------------------
use eframe::egui::{Color32, Rgba, Stroke};

#[derive(Clone, Copy, Debug)]
pub struct DisplayStyle {
    pub background_color: Color32,
    pub active_foreground_color: Color32,
    pub active_foreground_stroke: Stroke,
    pub inactive_foreground_color: Color32,
    pub inactive_foreground_stroke: Stroke,
}

impl DisplayStyle {
    #[must_use]
    pub fn foreground_color(&self, active: bool) -> Color32 {
        if active {
            self.active_foreground_color
        } else {
            self.inactive_foreground_color
        }
    }

    #[must_use]
    pub fn foreground_stroke(&self, active: bool) -> Stroke {
        if active {
            self.active_foreground_stroke
        } else {
            self.inactive_foreground_stroke
        }
    }

    #[must_use]
    pub fn foreground_color_blend(&self, value: f32) -> Color32 {
        Color32::from(eframe::egui::lerp(
            Rgba::from(self.inactive_foreground_color)..=Rgba::from(self.active_foreground_color),
            value,
        ))
    }

    #[must_use]
    pub fn foreground_stroke_blend(&self, value: f32) -> Stroke {
        Stroke::new(
            eframe::egui::lerp(
                self.inactive_foreground_stroke.width..=self.active_foreground_stroke.width,
                value,
            ),
            Color32::from(eframe::egui::lerp(
                Rgba::from(self.inactive_foreground_stroke.color)
                    ..=Rgba::from(self.active_foreground_stroke.color),
                value,
            )),
        )
    }

    #[must_use]
    pub fn system_style(ui: &eframe::egui::Ui) -> Self {
        let visuals = &ui.style().visuals;
        Self::from_visual(visuals)
    }

    fn from_visual(w: &eframe::egui::Visuals) -> Self {
        let w = &w.widgets;
        Self {
            background_color: w.active.bg_fill,
            active_foreground_color: w.active.text_color(),
            active_foreground_stroke: w.active.fg_stroke,
            inactive_foreground_color: w.inactive.text_color(),
            inactive_foreground_stroke: w.inactive.fg_stroke,
        }
    }
}

impl Default for DisplayStyle {
    fn default() -> Self {
        DisplayStylePreset::Default.style()
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, dyno_types::derive_more::Display, Eq, PartialEq)]
pub enum DisplayStylePreset {
    #[display(fmt = "Default")]
    Default,

    #[display(fmt = "Calculator")]
    Calculator,

    #[display(fmt = "Nintendo Game Boy")]
    NintendoGameBoy,

    #[display(fmt = "Knight Rider")]
    KnightRider,

    #[display(fmt = "Blue Negative")]
    BlueNegative,

    #[display(fmt = "Amber")]
    Amber,

    #[display(fmt = "Light Blue")]
    LightBlue,

    #[display(fmt = "DeLorean Red")]
    DeLoreanRed,

    #[display(fmt = "DeLorean Green")]
    DeLoreanGreen,

    #[display(fmt = "DeLorean Amber")]
    DeLoreanAmber,

    #[display(fmt = "Yamaha MU2000")]
    YamahaMU2000,
}

impl DisplayStylePreset {
    #[must_use]
    pub fn style(&self) -> DisplayStyle {
        match *self {
            DisplayStylePreset::Default => DisplayStyle {
                background_color: Color32::from_rgb(0x00, 0x20, 0x00),
                active_foreground_color: Color32::from_rgb(0x00, 0xF0, 0x00),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x00, 0x30, 0x00),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::Calculator => DisplayStyle {
                background_color: Color32::from_rgb(0xC5, 0xCB, 0xB6),
                active_foreground_color: Color32::from_rgb(0x00, 0x00, 0x00),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0xB9, 0xBE, 0xAB),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::NintendoGameBoy => DisplayStyle {
                background_color: Color32::from_rgb(0x9B, 0xBC, 0x0F),
                active_foreground_color: Color32::from_rgb(0x0F, 0x38, 0x0F),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x8B, 0xAC, 0x0F),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::KnightRider => DisplayStyle {
                background_color: Color32::from_rgb(0x10, 0x00, 0x00),
                active_foreground_color: Color32::from_rgb(0xC8, 0x00, 0x00),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x20, 0x00, 0x00),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::BlueNegative => DisplayStyle {
                background_color: Color32::from_rgb(0x00, 0x00, 0xFF),
                active_foreground_color: Color32::from_rgb(0xE0, 0xFF, 0xFF),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x28, 0x28, 0xFF),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::Amber => DisplayStyle {
                background_color: Color32::from_rgb(0x1D, 0x12, 0x07),
                active_foreground_color: Color32::from_rgb(0xFF, 0x9A, 0x21),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x33, 0x20, 0x00),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::LightBlue => DisplayStyle {
                background_color: Color32::from_rgb(0x0F, 0xB0, 0xBC),
                active_foreground_color: Color32::from_black_alpha(223),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_black_alpha(60),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::DeLoreanRed => DisplayStyle {
                background_color: Color32::from_rgb(0x12, 0x07, 0x0A),
                active_foreground_color: Color32::from_rgb(0xFF, 0x59, 0x13),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x48, 0x0A, 0x0B),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::DeLoreanGreen => DisplayStyle {
                background_color: Color32::from_rgb(0x05, 0x0A, 0x0A),
                active_foreground_color: Color32::from_rgb(0x4A, 0xF5, 0x0F),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x07, 0x29, 0x0F),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::DeLoreanAmber => DisplayStyle {
                background_color: Color32::from_rgb(0x08, 0x08, 0x0B),
                active_foreground_color: Color32::from_rgb(0xF2, 0xC4, 0x21),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x51, 0x2C, 0x0F),
                inactive_foreground_stroke: Stroke::NONE,
            },
            DisplayStylePreset::YamahaMU2000 => DisplayStyle {
                background_color: Color32::from_rgb(0x8C, 0xD7, 0x01),
                active_foreground_color: Color32::from_rgb(0x04, 0x4A, 0x00),
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: Color32::from_rgb(0x7B, 0xCE, 0x02),
                inactive_foreground_stroke: Stroke::NONE,
            },
        }
    }
}
