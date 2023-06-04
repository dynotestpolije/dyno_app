use dyno_core::derive_more::Display;

use eframe::egui::{pos2, Color32, Pos2, Rgba, Stroke, Vec2};
// ----------------------------------------------------------------------------
/// Anchor where to show toasts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// Top right corner.
    TopRight,
    /// Top left corner.
    TopLeft,
    /// Bottom right corner.
    BottomRight,
    /// Bottom left corner
    BottomLeft,
}

impl Anchor {
    #[inline]
    #[allow(unused)]
    pub(crate) fn anim_side(&self) -> f32 {
        match self {
            Anchor::TopRight | Anchor::BottomRight => 1.,
            Anchor::TopLeft | Anchor::BottomLeft => -1.,
        }
    }
}

impl Anchor {
    pub(crate) fn screen_corner(&self, sc: Pos2, margin: Vec2) -> Pos2 {
        let mut out = match self {
            Anchor::TopRight => pos2(sc.x, 0.),
            Anchor::TopLeft => pos2(0., 0.),
            Anchor::BottomRight => sc,
            Anchor::BottomLeft => pos2(0., sc.y),
        };
        self.apply_margin(&mut out, margin);
        out
    }

    pub(crate) fn apply_margin(&self, pos: &mut Pos2, margin: Vec2) {
        match self {
            Anchor::TopRight => {
                pos.x -= margin.x;
                pos.y += margin.y;
            }
            Anchor::TopLeft => {
                pos.x += margin.x;
                pos.y += margin.y
            }
            Anchor::BottomRight => {
                pos.x -= margin.x;
                pos.y -= margin.y;
            }
            Anchor::BottomLeft => {
                pos.x += margin.x;
                pos.y -= margin.y;
            }
        }
    }
}
// ----------------------------------------------------------------------------

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
#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    Eq,
    PartialEq,
    Default,
    dyno_core::serde::Serialize,
    dyno_core::serde::Deserialize,
)]
#[serde(crate = "dyno_core::serde")]
pub enum DisplayStylePreset {
    #[default]
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
    pub fn get_iter(self) -> core::array::IntoIter<Self, 11> {
        [
            Self::Default,
            Self::Calculator,
            Self::NintendoGameBoy,
            Self::KnightRider,
            Self::BlueNegative,
            Self::Amber,
            Self::LightBlue,
            Self::DeLoreanRed,
            Self::DeLoreanGreen,
            Self::DeLoreanAmber,
            Self::YamahaMU2000,
        ]
        .into_iter()
    }
}

impl DisplayStylePreset {
    #[must_use]
    pub fn style(&self) -> DisplayStyle {
        match *self {
            DisplayStylePreset::Default => DisplayStyle {
                background_color: crate::COLOR_BLUE_DYNO_DARKER,
                active_foreground_color: crate::COLOR_BLUE_DYNO,
                active_foreground_stroke: Stroke::NONE,
                inactive_foreground_color: crate::COLOR_BLUE_DYNO_DARK,
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

// ----------------------------------------------------------------------------
pub trait DynoWidgets: super::button::ButtonExt + Sized {
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

    fn combobox_from_slice<'s, V>(
        &mut self,
        label: impl Into<eframe::egui::WidgetText>,
        current_value: &mut V,
        values: &'s [V],
    ) -> eframe::egui::Response
    where
        V: Clone + Copy + PartialEq + dyno_core::AsStr<'s>;

    fn selectable_label_from_slice<W>(
        &mut self,
        slices: &'_ mut [bool],
        label: impl FnMut(usize) -> W,
    ) -> eframe::egui::Response
    where
        W: Into<eframe::egui::WidgetText>;

    fn toggle(&mut self, on: &mut bool) -> eframe::egui::Response;
    fn toggle_compact(&mut self, on: &mut bool) -> eframe::egui::Response;
}

impl DynoWidgets for eframe::egui::Ui {
    #[inline(always)]
    fn seven_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>,
    {
        use super::segment_display::SegmentedDisplay;
        self.add(SegmentedDisplay::seven_segment(value))
    }

    #[inline(always)]
    fn nine_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>,
    {
        use super::segment_display::SegmentedDisplay;
        self.add(SegmentedDisplay::seven_segment(value))
    }

    #[inline(always)]
    fn sixteen_segment<S>(&mut self, value: S) -> eframe::egui::Response
    where
        S: AsRef<str>,
    {
        use super::segment_display::SegmentedDisplay;
        self.add(SegmentedDisplay::seven_segment(value))
    }

    #[inline(always)]
    fn hyperlink_with_icon<U>(&mut self, url: U) -> eframe::egui::Response
    where
        U: AsRef<str>,
    {
        self.add(super::hyperlink::hyperlink_with_icon(url))
    }

    #[inline(always)]
    fn hyperlink_with_icon_to<S, U>(&mut self, label: S, url: U) -> eframe::egui::Response
    where
        S: AsRef<str>,
        U: AsRef<str>,
    {
        self.add(super::hyperlink::hyperlink_with_icon_to(label, url))
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
                match value.as_mut() {
                    Some(value) => {
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
    fn combobox_from_slice<'s, V>(
        &mut self,
        label: impl Into<eframe::egui::WidgetText>,
        current_value: &mut V,
        values: &'s [V],
    ) -> eframe::egui::Response
    where
        V: Clone + PartialEq + dyno_core::AsStr<'s>,
    {
        let combobox = eframe::egui::ComboBox::from_label(label)
            .selected_text(current_value.as_str())
            .show_ui(self, |ui| {
                values
                    .iter()
                    .map(|value| ui.selectable_value(current_value, value.clone(), value.as_str()))
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

    fn toggle(&mut self, on: &mut bool) -> eframe::egui::Response {
        let desired_size = self.spacing().interact_size.y * eframe::egui::vec2(2.0, 1.0);
        let (rect, mut response) =
            self.allocate_exact_size(desired_size, eframe::egui::Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed(); // report back that the value changed
        }
        response.widget_info(|| {
            eframe::egui::WidgetInfo::selected(eframe::egui::WidgetType::Checkbox, *on, "")
        });
        if self.is_rect_visible(rect) {
            let how_on = self.ctx().animate_bool(response.id, *on);
            let visuals = self.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            self.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x =
                eframe::egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = eframe::egui::pos2(circle_x, rect.center().y);
            self.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }
        response
    }
    fn toggle_compact(&mut self, on: &mut bool) -> eframe::egui::Response {
        let desired_size = self.spacing().interact_size.y * eframe::egui::vec2(2.0, 1.0);
        let (rect, mut response) =
            self.allocate_exact_size(desired_size, eframe::egui::Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response.widget_info(|| {
            eframe::egui::WidgetInfo::selected(eframe::egui::WidgetType::Checkbox, *on, "")
        });

        if self.is_rect_visible(rect) {
            let how_on = self.ctx().animate_bool(response.id, *on);
            let visuals = self.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            self.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x =
                eframe::egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = eframe::egui::pos2(circle_x, rect.center().y);
            self.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }

        response
    }
}

pub trait PainterHelper {
    fn text_center_monospace(
        &self,
        pos: impl Into<eframe::emath::Pos2>,
        text: impl ToString,
        text_size: f32,
        color: impl Into<Color32>,
        wrap_width: Option<f32>,
    );
    fn text_center_prop(
        &self,
        pos: impl Into<eframe::emath::Pos2>,
        text: impl ToString,
        text_size: f32,
        color: impl Into<Color32>,
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
        color: impl Into<Color32>,
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
        color: impl Into<Color32>,
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
