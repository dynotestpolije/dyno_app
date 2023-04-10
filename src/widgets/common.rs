use eframe::{
    egui::*,
    emath::{almost_equal, Rot2},
    epaint::TextShape,
};

use dyno_types::derive_more::Display;
use itertools::Itertools;
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum Orientation {
    #[display(fmt = "Top")]
    Top,

    #[display(fmt = "Bottom")]
    Bottom,

    #[display(fmt = "Left")]
    Left,

    #[display(fmt = "Right")]
    Right,

    Custom(f32),
}

impl Orientation {
    #[inline]
    #[allow(unused)]
    pub(crate) fn rot2(&self) -> Rot2 {
        match *self {
            Self::Right => Rot2::from_angle(std::f32::consts::TAU * 0.00),
            Self::Bottom => Rot2::from_angle(std::f32::consts::TAU * 0.25),
            Self::Left => Rot2::from_angle(std::f32::consts::TAU * 0.50),
            Self::Top => Rot2::from_angle(std::f32::consts::TAU * 0.75),
            Self::Custom(angle) => Rot2::from_angle(angle),
        }
    }
}
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

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Winding {
    #[display(fmt = "Clockwise")]
    Clockwise,

    #[display(fmt = "Counter-clockwise")]
    Counterclockwise,
}

impl Winding {
    #[allow(unused)]
    #[inline(always)]
    pub(crate) const fn to_float(self) -> f32 {
        match self {
            Self::Clockwise => 1.0,
            Self::Counterclockwise => -1.0,
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum WrapMode {
    #[display(fmt = "None")]
    None,

    #[display(fmt = "Signed")]
    Signed,

    #[display(fmt = "Unsigned")]
    Unsigned,
}

// ----------------------------------------------------------------------------

#[non_exhaustive]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum WidgetShape {
    #[display(fmt = "Circle")]
    Circle,

    #[display(fmt = "Square")]
    Square,

    #[display(fmt = "Squircle")]
    Squircle(f32),

    #[display(fmt = "Polygon")]
    Polygon(usize),

    #[display(fmt = "SuperPolygon")]
    SuperPolygon(usize, f32),

    #[display(fmt = "Rotated")]
    Rotated(Box<WidgetShape>, f32),

    #[display(fmt = "Scaled")]
    Scaled(Box<WidgetShape>, f32),

    #[display(fmt = "Mix")]
    Mix(Box<WidgetShape>, Box<WidgetShape>, f32),

    #[display(fmt = "Mininum")]
    Min(Box<WidgetShape>, Box<WidgetShape>),

    #[display(fmt = "Maximum")]
    Max(Box<WidgetShape>, Box<WidgetShape>),
}

impl WidgetShape {
    const RESOLUTION: usize = 32;

    pub(crate) fn eval(&self, theta: f32) -> f32 {
        match self {
            WidgetShape::Circle => 1.0,
            WidgetShape::Square => (1.0 / theta.cos().abs()).min(1.0 / theta.sin().abs()),
            WidgetShape::Squircle(factor) => {
                assert!(*factor > 0.0, "squircle factor must be positive");
                let a = theta.cos().abs().powf(*factor);
                let b = theta.sin().abs().powf(*factor);
                (a + b).powf(-1.0 / *factor)
            }
            WidgetShape::Polygon(n) => {
                assert!(*n >= 3, "polygon must have at least 3 sides");
                1.0 / ((*n as f32 / 2.0 * theta).cos().asin() * 2.0 / *n as f32).cos()
            }
            WidgetShape::SuperPolygon(n, factor) => {
                assert!(*n >= 3, "polygon must have at least 3 sides");
                assert!(*factor > 0.0, "polygon factor must be positive");
                assert!(
                    (0.0..=2.0).contains(factor),
                    "polygon factor must be between 0.0 and 2.0"
                );

                // https://mathworld.wolfram.com/Superellipse.html
                let a = (0.25 * (*n as f32) * theta).cos().abs().powf(*factor);
                let b = (0.25 * (*n as f32) * theta).sin().abs().powf(*factor);
                (a + b).powf(-1.0 / *factor)
            }
            WidgetShape::Rotated(shape, rotation) => shape.eval(theta - rotation),
            WidgetShape::Scaled(shape, scale) => shape.eval(theta) * scale,
            WidgetShape::Mix(shape_a, shape_b, t) => {
                (shape_a.eval(theta) * (1.0 - t)) + (shape_b.eval(theta) * t)
            }
            WidgetShape::Min(shape_a, shape_b) => shape_a.eval(theta).min(shape_b.eval(theta)),
            WidgetShape::Max(shape_a, shape_b) => shape_a.eval(theta).max(shape_b.eval(theta)),
        }
    }

    #[allow(unused)]
    pub(crate) fn paint_shape(
        &self,
        ui: &mut Ui,
        center: Pos2,
        radius: f32,
        fill: Color32,
        stroke: Stroke,
        rotation: Rot2,
    ) {
        let outline_points = (0..Self::RESOLUTION)
            .map(move |i| {
                let angle = (i as f32 / Self::RESOLUTION as f32) * std::f32::consts::TAU;
                let shape_radius = self.eval(angle - (rotation * Vec2::RIGHT).angle());
                center + Vec2::angled(angle) * radius * shape_radius
            })
            .collect_vec();

        // https://github.com/emilk/egui/issues/513
        outline_points
            .iter()
            .circular_tuple_windows()
            .for_each(|(point_1, point_2)| {
                ui.painter().add(Shape::convex_polygon(
                    vec![center, *point_1, *point_2],
                    fill,
                    Stroke::new(1.0, fill),
                ));
            });

        ui.painter().add(Shape::closed_line(outline_points, stroke));
    }
    #[allow(unused)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint_arc(
        &self,
        ui: &mut Ui,
        center: Pos2,
        inner_radius: f32,
        outer_radius: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Color32,
        stroke: Stroke,
        rotation: Rot2,
    ) {
        // NOTE: convex_polygon() is broken, spews rendering artifacts all over
        //   the window when it tries to render degenerate polygons:
        //     ∃(P1,P2) ∈ Poly (dist(P1,P2) ≈ 0)

        // HACK: convex_polygon() workaround
        if almost_equal(start_angle, end_angle, 0.001) {
            let shape_radius = self.eval(start_angle - (rotation * Vec2::RIGHT).angle());

            ui.painter().add(Shape::line_segment(
                [
                    center + Vec2::angled(start_angle) * inner_radius * shape_radius,
                    center + Vec2::angled(start_angle) * outer_radius * shape_radius,
                ],
                stroke,
            ));
            return;
        }

        let generate_arc_points = |radius| {
            (0..=Self::RESOLUTION).map(move |i| {
                let angle = lerp(start_angle..=end_angle, i as f32 / Self::RESOLUTION as f32);
                let shape_radius = self.eval(angle - (rotation * Vec2::RIGHT).angle());
                center + Vec2::angled(angle) * radius * shape_radius
            })
        };

        // HACK: convex_polygon() workaround
        let inner_radius = inner_radius.max(0.1);

        let outer_arc = generate_arc_points(outer_radius).collect::<Vec<_>>();
        let inner_arc = generate_arc_points(inner_radius).collect::<Vec<_>>();

        // https://github.com/emilk/egui/issues/513
        outer_arc
            .iter()
            .zip(inner_arc.iter())
            .tuple_windows()
            .for_each(|((outer_1, inner_1), (outer_2, inner_2))| {
                ui.painter().add(Shape::convex_polygon(
                    vec![*outer_1, *inner_1, *inner_2, *outer_2],
                    fill,
                    Stroke::new(1.0, fill),
                ));
            });

        let outline_points: Vec<Pos2> = outer_arc
            .iter()
            .chain(inner_arc.iter().rev())
            .copied()
            .collect();

        ui.painter().add(Shape::closed_line(outline_points, stroke));

        // TODO: Remove hacks and paint the arc with a single call:
        // Shape::concave_polygon(
        //     outline_points, // outer_arc.chain(inner_arc.rev())
        //     fill,
        //     stroke,
        // )
    }
}

// ----------------------------------------------------------------------------

#[allow(unused)]
pub(crate) fn paint_ellipse(
    ui: &mut Ui,
    center: Pos2,
    size: Vec2,
    fill: Color32,
    stroke: Stroke,
    rotation: Rot2,
) {
    const ELLIPSE_RESOLUTION: usize = 32;

    let points = (0..ELLIPSE_RESOLUTION)
        .map(|i| ((i as f32) / (ELLIPSE_RESOLUTION as f32)) * std::f32::consts::TAU)
        .map(|t| center + rotation * (Vec2::angled(t) * (size / 2.0)))
        .collect();

    ui.painter()
        .add(Shape::convex_polygon(points, fill, stroke));
}

// ----------------------------------------------------------------------------
#[allow(unused)]
#[inline(always)]
pub fn map_range<T: dyno_types::Numeric>(
    value: T,
    in_min: T,
    in_max: T,
    out_min: T,
    out_max: T,
) -> T {
    ((value - in_min) / (in_max - in_min)) * (out_max - out_min) + out_min
}

#[allow(unused)]
#[inline(always)]
pub fn value_to_radians(val: f32, min: f32, max: f32, min_degree: f32, max_degree: f32) -> f32 {
    (((val - min) / (max - min)) * (max_degree - min_degree) + min_degree).to_radians()
}

#[allow(unused)]
#[inline(always)]
pub(crate) fn constrain_angle(prev_value: f32, new_value: f32, min: f32, max: f32) -> f32 {
    let new_value =
        new_value + ((prev_value / std::f32::consts::TAU).round() * std::f32::consts::TAU);
    if new_value - prev_value > (std::f32::consts::TAU / 2.0) {
        (new_value - std::f32::consts::TAU).min(min).max(max)
    } else {
        (new_value + std::f32::consts::TAU).min(min).max(max)
    }
}

#[allow(unused)]
pub(crate) fn snap_wrap_constrain_angle(
    prev_value: f32,
    mut new_value: f32,
    snap: Option<f32>,
    wrap: WrapMode,
    min: Option<f32>,
    max: Option<f32>,
) -> f32 {
    if let Some(snap_angle) = snap {
        assert!(
            snap_angle > 0.0,
            "non-positive snap angles are not supported"
        );
        new_value = (new_value / snap_angle).round() * snap_angle;
    }

    if wrap == WrapMode::Unsigned {
        new_value = normalized_angle_unsigned_excl(new_value);
    }

    if wrap == WrapMode::None {
        let prev_turns = (prev_value / std::f32::consts::TAU).round();
        new_value += prev_turns * std::f32::consts::TAU;

        if new_value - prev_value > (std::f32::consts::TAU / 2.0) {
            new_value -= std::f32::consts::TAU;
        } else if new_value - prev_value < -(std::f32::consts::TAU / 2.0) {
            new_value += std::f32::consts::TAU;
        }
    }

    if let Some(min) = min {
        new_value = new_value.max(min);
    }

    if let Some(max) = max {
        new_value = new_value.min(max);
    }

    new_value
}

// ----------------------------------------------------------------------------

#[allow(unused)]
/// Wrap angle to `(0..std::f32::consts::TAU)` range.
pub(crate) fn normalized_angle_unsigned_excl(angle: f32) -> f32 {
    ((angle % std::f32::consts::TAU) + std::f32::consts::TAU) % std::f32::consts::TAU
}
#[allow(unused)]
/// Wrap angle to `(0..=std::f32::consts::TAU)` range.
pub(crate) fn normalized_angle_unsigned_incl(angle: f32) -> f32 {
    if angle < 0.0 {
        ((angle % std::f32::consts::TAU) + std::f32::consts::TAU) % std::f32::consts::TAU
    } else if angle > std::f32::consts::TAU {
        angle % std::f32::consts::TAU
    } else {
        angle
    }
}

// ----------------------------------------------------------------------------

pub(crate) trait SymLog {
    fn symlog(&self, base: Self) -> Self;
}

impl SymLog for f32 {
    fn symlog(&self, base: Self) -> Self {
        if self.abs() < base {
            (self.abs() / base) * self.signum()
        } else {
            self.abs().log(base) * self.signum()
        }
    }
}

// ----------------------------------------------------------------------------
// TODO: Remove this trait when egui exposes text rotation with a sane API.

pub trait RotatedText {
    fn rotated_text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
        angle: f32,
    ) -> Rect;
}

impl RotatedText for Painter {
    fn rotated_text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
        angle: f32,
    ) -> Rect {
        let galley = self.layout_no_wrap(text.to_string(), font_id, text_color);
        let rect = anchor.anchor_rect(Rect::from_min_size(pos, galley.size()));

        let half_size = galley.size() / 2.0;

        self.add(TextShape {
            angle,
            override_text_color: Some(text_color),
            ..TextShape::new(
                pos - Rot2::from_angle(angle) * (half_size + (anchor.to_sign() * half_size)),
                galley,
            )
        });

        rect
    }
}
