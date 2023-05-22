use dyno_core::Numeric;
use eframe::egui::*;
use eframe::emath::Rot2;

#[derive(Debug, Clone)]
pub struct Gauges {
    value: f32,
    diameter: Option<f32>,

    types: GaugeTypes,
    animated: bool,
}
impl Gauges {
    pub fn new(preset: GaugeTypes, value: f32) -> Self {
        Self {
            value,
            types: preset,
            diameter: None,
            animated: false,
        }
    }

    pub fn speed(value: impl Numeric) -> Self {
        Self::new(GaugeTypes::SpeedGauge, value.to_f32()).animated(true)
    }

    pub fn rpm_roda(value: impl Numeric) -> Self {
        // map value from 1000 to 1 ( value *  1000 )
        Self::new(GaugeTypes::RpmRodaGauge, value.to_f32() * 0.001).animated(true)
    }
    pub fn rpm_engine(value: impl Numeric) -> Self {
        // map value from 1000 to 1 ( value *  1000 )
        Self::new(GaugeTypes::RpmEngineGauge, value.to_f32() * 0.001).animated(true)
    }

    pub fn torque(value: impl Numeric) -> Self {
        Self::new(GaugeTypes::TorqueGauge, value.to_f32()).animated(true)
    }

    pub fn horsepower(value: impl Numeric) -> Self {
        Self::new(GaugeTypes::HorsepowerGauge, value.to_f32()).animated(true)
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = Some(diameter);
        self
    }
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl Widget for Gauges {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            value,
            diameter,
            types,
            animated: _,
        } = self;
        let desired_size = match diameter {
            Some(diameter) => Vec2::splat(diameter),
            None => Vec2::splat(ui.available_size().min_elem()),
        };
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let center = rect.center();
            let diameter = rect.size().min_elem();
            let radius = diameter * 0.5;

            let GaugePreset {
                needle_color,
                min,
                max,
                min_degree,
                max_degree,
                foreground_color: _,
            } = types.presets(ui.visuals());

            let value_degree = ui.ctx().animate_value_with_time(
                response.id,
                (((value - min) / (max - min)) * (max_degree - min_degree) + min_degree)
                    .clamp(min_degree, max_degree),
                ui.style().animation_time,
            );

            let painter = GaugeBG::new(self.types).draw(ui, rect, radius);
            GaugeNeedle::new(value_degree, center, radius, needle_color).draw(&painter);
        }
        response.on_hover_text(self.types.to_string())
    }
}

// ---------------------------------------------------------------- //

#[derive(Debug, Clone)]
struct GaugeNeedle {
    vec: Vec2,
    center: Pos2,
    color: Color32,
    radius: f32,
}
impl GaugeNeedle {
    // IS MAGIC, dont touch it!
    const ROT: Rot2 = Rot2 {
        s: 0.125_333_23_f32,
        c: 0.992_114_7_f32,
    };

    fn new(value: f32, center: Pos2, radius: f32, color: Color32) -> Self {
        let radius = radius * 0.8;
        Self {
            color,
            center,
            radius: radius * 0.1,
            vec: Rot2::from_angle(value.to_radians()) * Vec2 { x: 0f32, y: radius },
        }
    }

    fn draw(self, painter: &Painter) {
        let Self {
            vec,
            center,
            color,
            radius,
        } = self;

        let length = vec.length() / 4f32;
        let dir = vec.normalized();
        let tip = center + vec;
        let points = vec![
            self.center + self.vec,
            tip - (length * 0.3) * (Self::ROT * dir),
            self.center - length * (Self::ROT * dir),
            self.center - length * (Self::ROT.inverse() * dir),
            tip - (length * 0.3) * (Self::ROT.inverse() * dir),
        ];

        painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
        painter.circle_filled(center, radius, Color32::BLACK);
    }
}

enum GaugeBG<'a> {
    Image(&'a super::images::Img),
    Color(Color32),
}

impl GaugeBG<'_> {
    fn new(types: GaugeTypes) -> Self {
        use crate::assets::{
            COLORIMAGE_GAUGE_HP, COLORIMAGE_GAUGE_RPM, COLORIMAGE_GAUGE_SPEED,
            COLORIMAGE_GAUGE_TORQUE,
        };
        match types {
            GaugeTypes::Default => GaugeBG::Color(Color32::from_black_alpha(200)),
            GaugeTypes::RpmRodaGauge => GaugeBG::Image(&COLORIMAGE_GAUGE_RPM),
            GaugeTypes::RpmEngineGauge => GaugeBG::Image(&COLORIMAGE_GAUGE_RPM),
            GaugeTypes::SpeedGauge => GaugeBG::Image(&COLORIMAGE_GAUGE_SPEED),
            GaugeTypes::TorqueGauge => GaugeBG::Image(&COLORIMAGE_GAUGE_TORQUE),
            GaugeTypes::HorsepowerGauge => GaugeBG::Image(&COLORIMAGE_GAUGE_HP),
        }
    }

    #[inline]
    fn draw(&self, ui: &mut Ui, rect: Rect, rad: f32) -> Painter {
        let paint = ui.painter_at(rect);
        match self {
            Self::Image(img) => paint.add(img.get_shape(ui, rect)),
            Self::Color(c) => paint.add(epaint::CircleShape {
                center: rect.center(),
                radius: rad,
                fill: *c,
                stroke: Stroke::new(2f32, *c),
            }),
        };
        paint
    }
}

pub struct GaugePreset {
    pub needle_color: Color32,
    pub foreground_color: Color32,
    pub min: f32,
    pub max: f32,
    pub min_degree: f32,
    pub max_degree: f32,
}

impl GaugePreset {
    #[inline(always)]
    const fn from_system(vis: &Visuals) -> Self {
        let needle_color = vis.widgets.active.fg_stroke.color;
        Self {
            needle_color,
            foreground_color: needle_color,
            min: 0f32,
            max: 200f32,
            min_degree: 0f32,
            max_degree: 270f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeTypes {
    Default,
    RpmRodaGauge,
    RpmEngineGauge,
    SpeedGauge,
    TorqueGauge,
    HorsepowerGauge,
}
impl GaugeTypes {
    #[inline(always)]
    pub const fn presets(&self, vis: &Visuals) -> GaugePreset {
        match self {
            GaugeTypes::Default => GaugePreset::from_system(vis),
            GaugeTypes::RpmRodaGauge => GaugePreset {
                needle_color: Color32::from_rgb(0, 204, 255),
                foreground_color: Color32::from_rgb(85, 221, 255),
                min: 0f32,
                max: 15f32,
                min_degree: 0f32,
                max_degree: 270f32,
            },
            GaugeTypes::RpmEngineGauge => GaugePreset {
                needle_color: Color32::from_rgb(0, 204, 255),
                foreground_color: Color32::from_rgb(85, 221, 255),
                min: 0f32,
                max: 15f32,
                min_degree: 0f32,
                max_degree: 270f32,
            },
            GaugeTypes::SpeedGauge => GaugePreset {
                needle_color: Color32::from_rgb(0, 204, 255),
                foreground_color: Color32::from_rgb(85, 221, 255),
                min: 0f32,
                max: 240f32,
                min_degree: 50f32,
                max_degree: 310f32,
            },
            GaugeTypes::TorqueGauge => GaugePreset {
                needle_color: Color32::from_rgb(0, 204, 255),
                foreground_color: Color32::from_rgb(85, 221, 255),
                min: 0f32,
                max: 100f32,
                min_degree: 50f32,
                max_degree: 310f32,
            },
            GaugeTypes::HorsepowerGauge => GaugePreset {
                needle_color: Color32::from_rgb(0, 204, 255),
                foreground_color: Color32::from_rgb(85, 221, 255),
                min: 0f32,
                max: 100f32,
                min_degree: 50f32,
                max_degree: 310f32,
            },
        }
    }
}

impl std::fmt::Display for GaugeTypes {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GaugeTypes::Default => f.write_str("Gauge Widget"),
            GaugeTypes::RpmRodaGauge => f.write_str("Rpm Roda Gauge"),
            GaugeTypes::RpmEngineGauge => f.write_str("Rpm Engine Gauge"),
            GaugeTypes::SpeedGauge => f.write_str("Speed Gauge"),
            GaugeTypes::TorqueGauge => f.write_str("Torque Gauge"),
            GaugeTypes::HorsepowerGauge => f.write_str("Torque Gauge"),
        }
    }
}
