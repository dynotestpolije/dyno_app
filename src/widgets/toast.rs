use eframe::egui::{
    pos2, vec2, Color32, Context, FontId, Id, LayerId, Order, Pos2, Rect, Rounding, Stroke, Vec2,
};

use super::Anchor;

const TOAST_WIDTH: f32 = 180.;
const TOAST_HEIGHT: f32 = 34.;

const ERROR_COLOR: Color32 = Color32::from_rgb(200, 90, 90);
const ERROR_COLOR_BG: Color32 = Color32::from_rgb(20, 9, 9);

const INFO_COLOR: Color32 = Color32::from_rgb(150, 200, 210);
const INFO_COLOR_BG: Color32 = Color32::from_rgb(15, 20, 21);

const WARNING_COLOR: Color32 = Color32::from_rgb(230, 220, 140);
const WARNING_COLOR_BG: Color32 = Color32::from_rgb(23, 22, 14);

const SUCCESS_COLOR: Color32 = Color32::from_rgb(140, 230, 140);
const SUCCESS_COLOR_BG: Color32 = Color32::from_rgb(14, 23, 14);

/// Level of importance
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ToastLevel {
    #[default]
    Info,
    Warning,
    Error,
    Success,
    None,
}

#[derive(Debug)]
pub enum ToastState {
    Appear,
    Disapper,
    Disappeared,
    Idle,
}

impl ToastState {
    pub fn appearing(&self) -> bool {
        matches!(self, Self::Appear)
    }
    pub fn disappearing(&self) -> bool {
        matches!(self, Self::Disapper)
    }
    pub fn disappeared(&self) -> bool {
        matches!(self, Self::Disappeared)
    }
    pub fn idling(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

/// Container for options for initlizing toasts
pub struct ToastOptions {
    duration: Option<std::time::Duration>,
    level: ToastLevel,
    closable: bool,
}

/// Single notification or *toast*
#[derive(Debug)]
pub struct Toast {
    pub(crate) level: ToastLevel,
    pub(crate) caption: String,
    // (initial, current)
    pub(crate) duration: Option<(f32, f32)>,
    pub(crate) height: f32,
    pub(crate) width: f32,
    pub(crate) closable: bool,

    pub(crate) state: ToastState,
    pub(crate) value: f32,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            duration: Some(std::time::Duration::from_millis(3500)),
            level: ToastLevel::None,
            closable: true,
        }
    }
}

fn duration_to_seconds_f32(duration: std::time::Duration) -> f32 {
    duration.as_nanos() as f32 * 1e-9
}

impl Toast {
    fn new(caption: impl ToString, options: ToastOptions) -> Self {
        Self {
            caption: caption.to_string(),
            height: TOAST_HEIGHT,
            width: TOAST_WIDTH,
            duration: if let Some(dur) = options.duration {
                let max_dur = duration_to_seconds_f32(dur);
                Some((max_dur, max_dur))
            } else {
                None
            },
            closable: options.closable,
            level: options.level,

            value: 0.,
            state: ToastState::Appear,
        }
    }

    /// Creates new basic toast, can be closed by default.
    pub fn basic(caption: impl ToString) -> Self {
        Self::new(caption, ToastOptions::default())
    }

    /// Creates new success toast, can be closed by default.
    pub fn success(caption: impl ToString) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Success,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new info toast, can be closed by default.
    pub fn info(caption: impl ToString) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Info,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new warning toast, can be closed by default.
    pub fn warning(caption: impl ToString) -> Self {
        Self::new(
            caption,
            ToastOptions {
                level: ToastLevel::Warning,
                ..ToastOptions::default()
            },
        )
    }

    /// Creates new error toast, can not be closed by default.
    pub fn error(caption: impl ToString) -> Self {
        Self::new(
            caption,
            ToastOptions {
                closable: false,
                level: ToastLevel::Error,
                ..ToastOptions::default()
            },
        )
    }

    /// Set the options with a ToastOptions
    pub fn set_options(&mut self, options: ToastOptions) -> &mut Self {
        self.set_closable(options.closable);
        self.set_duration(options.duration);
        self.set_level(options.level);
        self
    }

    /// Change the level of the toast
    pub fn set_level(&mut self, level: ToastLevel) -> &mut Self {
        self.level = level;
        self
    }

    /// Can use close the toast?
    pub fn set_closable(&mut self, closable: bool) -> &mut Self {
        self.closable = closable;
        self
    }

    /// In what time should the toast expire? Set to `None` for no expiry.
    pub fn set_duration(&mut self, duration: Option<std::time::Duration>) -> &mut Self {
        if let Some(duration) = duration {
            let max_dur = duration_to_seconds_f32(duration);
            self.duration = Some((max_dur, max_dur));
        } else {
            self.duration = None;
        }
        self
    }

    /// Toast's box height
    pub fn set_height(&mut self, height: f32) -> &mut Self {
        self.height = height;
        self
    }

    /// Toast's box width
    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }

    pub fn duration_timeout_check(&mut self) {
        if let Some((_initial_d, current_d)) = self.duration {
            if current_d <= 0. {
                self.state = ToastState::Disapper;
            }
        }
    }

    /// Dismiss this toast
    pub fn dismiss(&mut self) {
        self.state = ToastState::Disapper;
    }

    pub(crate) fn calc_anchored_rect(&self, pos: Pos2, anchor: Anchor) -> Rect {
        match anchor {
            Anchor::TopRight => Rect {
                min: pos2(pos.x - self.width, pos.y),
                max: pos2(pos.x, pos.y + self.height),
            },
            Anchor::TopLeft => Rect {
                min: pos,
                max: pos + vec2(self.width, self.height),
            },
            Anchor::BottomRight => Rect {
                min: pos - vec2(self.width, self.height),
                max: pos,
            },
            Anchor::BottomLeft => Rect {
                min: pos2(pos.x, pos.y - self.height),
                max: pos2(pos.x + self.width, pos.y),
            },
        }
    }

    pub(crate) fn adjust_next_pos(&self, pos: &mut Pos2, anchor: Anchor, spacing: f32) {
        if matches!(anchor, Anchor::TopRight | Anchor::TopLeft) {
            pos.y += self.height + spacing
        } else {
            pos.y -= self.height + spacing
        }
    }
}
pub struct Toasts {
    toasts: Vec<Toast>,
    anchor: Anchor,
    margin: Vec2,
    spacing: f32,
    padding: Vec2,
    reverse: bool,
    speed: f32,

    held: bool,
}

impl Toasts {
    /// Creates new [`Toasts`] instance.
    pub const fn new() -> Self {
        Self {
            anchor: Anchor::TopRight,
            margin: vec2(8., 8.),
            toasts: vec![],
            spacing: 8.,
            padding: vec2(10., 10.),
            held: false,
            speed: 4.,
            reverse: false,
        }
    }

    /// Adds new toast to the collection.
    /// By default adds toast at the end of the list, can be changed with `self.reverse`.
    pub fn add(&mut self, toast: Toast) {
        if self.reverse {
            return self.toasts.insert(0, toast);
        }
        self.toasts.push(toast);
    }

    /// Dismisses the oldest toast
    pub fn dismiss_oldest_toast(&mut self) {
        if let Some(toast) = self.toasts.first_mut() {
            toast.dismiss();
        }
    }

    /// Dismisses the most recent toast
    pub fn dismiss_latest_toast(&mut self) {
        if let Some(toast) = self.toasts.last_mut() {
            toast.dismiss();
        }
    }

    /// Dismisses all toasts
    pub fn dismiss_all_toasts(&mut self) {
        self.toasts.iter_mut().for_each(|toast| toast.dismiss());
    }

    /// Shortcut for adding a toast with info `success`.
    #[inline]
    pub fn success(&mut self, caption: impl ToString + std::fmt::Display) {
        dyno_core::log::info!("{}", &caption);
        self.add(Toast::success(caption))
    }

    /// Shortcut for adding a toast with info `level`.
    #[inline]
    pub fn info(&mut self, caption: impl ToString + std::fmt::Display) {
        let caption = caption.to_string();
        dyno_core::log::info!("{}", &caption);
        self.add(Toast::info(caption))
    }

    /// Shortcut for adding a toast with warning `level`.
    #[inline]
    pub fn warning(&mut self, caption: impl ToString + std::fmt::Display) {
        let caption = caption.to_string();
        dyno_core::log::warn!("{}", &caption);
        self.add(Toast::warning(caption))
    }

    /// Shortcut for adding a toast with error `level`.
    #[inline]
    pub fn error(&mut self, caption: impl ToString + std::fmt::Display) {
        let caption = caption.to_string();
        dyno_core::log::error!("{}", &caption);
        self.add(Toast::error(caption))
    }

    /// Shortcut for adding a toast with no level.
    #[inline]
    pub fn basic(&mut self, caption: impl ToString) {
        self.add(Toast::basic(caption))
    }

    /// Should toasts be added in reverse order?
    pub const fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Where toasts should appear.
    pub const fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets spacing between adjacent toasts.
    pub const fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Margin or distance from screen to toasts' bounding boxes
    pub const fn with_margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Padding or distance from toasts' bounding boxes to inner contents.
    pub const fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }
}

impl Toasts {
    /// Displays toast queue
    pub fn show(&mut self, ctx: &Context) {
        let Self {
            anchor,
            margin,
            spacing,
            padding,
            toasts,
            held,
            speed,
            ..
        } = self;

        let pos_max_rect = ctx.input(|input| input.screen_rect.max);
        let mut pos = anchor.screen_corner(pos_max_rect, *margin);
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("dyno_toasts")));

        let mut dismiss = None;

        // Remove disappeared toasts
        toasts.retain(|toast| !toast.state.disappeared());
        // Start disappearing expired toasts
        toasts
            .iter_mut()
            .for_each(|tst| tst.duration_timeout_check());
        // `held` used to prevent sticky removal
        ctx.input(|inp| *held = !inp.pointer.primary_released());

        let visuals = ctx.style().visuals.widgets.noninteractive;
        let mut update = false;

        for (i, toast) in toasts.iter_mut().enumerate() {
            // Decrease duration if idling
            if let Some((_, d)) = toast.duration.as_mut() {
                if toast.state.idling() {
                    *d -= ctx.input(|inp| inp.stable_dt);
                    update = true;
                }
            }
            let bg_color = match toast.level {
                ToastLevel::Info => INFO_COLOR_BG,
                ToastLevel::Warning => WARNING_COLOR_BG,
                ToastLevel::Error => ERROR_COLOR_BG,
                ToastLevel::Success => SUCCESS_COLOR_BG,
                ToastLevel::None => visuals.bg_fill,
            };
            let fg_color = if toast.level == ToastLevel::None {
                visuals.fg_stroke.color
            } else {
                Color32::WHITE
            };

            // Create toast label
            let caption_galley = ctx.fonts(|font| {
                font.layout(
                    toast.caption.clone(),
                    FontId::proportional(14.),
                    fg_color,
                    f32::INFINITY,
                )
            });

            let (caption_width, caption_height) =
                (caption_galley.rect.width(), caption_galley.rect.height());

            let line_count = toast.caption.chars().filter(|c| *c == '\n').count() + 1;
            let icon_width = caption_height / line_count as f32;

            // Create toast icon
            let icon_font = FontId::proportional(icon_width);
            let icon_galley = ctx.fonts(|font| match toast.level {
                ToastLevel::Info => {
                    Some(font.layout("ℹ".into(), icon_font, INFO_COLOR, f32::INFINITY))
                }
                ToastLevel::Warning => {
                    Some(font.layout("⚠".into(), icon_font, WARNING_COLOR, f32::INFINITY))
                }
                ToastLevel::Error => {
                    Some(font.layout("！".into(), icon_font, ERROR_COLOR, f32::INFINITY))
                }
                ToastLevel::Success => {
                    Some(font.layout("✅".into(), icon_font, SUCCESS_COLOR, f32::INFINITY))
                }
                ToastLevel::None => None,
            });

            let (action_width, action_height) = if let Some(icon_galley) = icon_galley.as_ref() {
                (icon_galley.rect.width(), icon_galley.rect.height())
            } else {
                (0., 0.)
            };

            // Create closing cross
            let cross_galley = if toast.closable {
                let cross_fid = FontId::proportional(icon_width);
                let cross_galley =
                    ctx.fonts(|font| font.layout("❌".into(), cross_fid, fg_color, f32::INFINITY));
                Some(cross_galley)
            } else {
                None
            };

            let (cross_width, cross_height) = if let Some(cross_galley) = cross_galley.as_ref() {
                (cross_galley.rect.width(), cross_galley.rect.height())
            } else {
                (0., 0.)
            };

            let icon_x_padding = (0., 7.);
            let cross_x_padding = (7., 0.);

            let icon_width_padded = if icon_width == 0. {
                0.
            } else {
                icon_width + icon_x_padding.0 + icon_x_padding.1
            };
            let cross_width_padded = if cross_width == 0. {
                0.
            } else {
                cross_width + cross_x_padding.0 + cross_x_padding.1
            };

            toast.width = icon_width_padded + caption_width + cross_width_padded + (padding.x * 2.);
            toast.height = action_height.max(caption_height).max(cross_height) + padding.y * 2.;

            let anim_offset = toast.width * (1. - ease_in_cubic(toast.value));
            pos.x += anim_offset * anchor.anim_side();
            let rect = toast.calc_anchored_rect(pos, *anchor);

            // Required due to positioning of the next toast
            pos.x -= anim_offset * anchor.anim_side();

            // Draw background
            painter.rect_filled(rect, Rounding::same(4.), bg_color);

            // Paint icon
            if let Some((icon_galley, true)) =
                icon_galley.zip(Some(toast.level != ToastLevel::None))
            {
                let oy = toast.height / 2. - action_height / 2.;
                let ox = padding.x + icon_x_padding.0;
                painter.galley(rect.min + vec2(ox, oy), icon_galley);
            }

            // Paint caption
            let oy = toast.height / 2. - caption_height / 2.;
            let o_from_icon = if action_width == 0. {
                0.
            } else {
                action_width + icon_x_padding.1
            };
            let o_from_cross = if cross_width == 0. {
                0.
            } else {
                cross_width + cross_x_padding.0
            };
            let ox = (toast.width / 2. - caption_width / 2.) + o_from_icon / 2. - o_from_cross / 2.;
            painter.galley(rect.min + vec2(ox, oy), caption_galley);

            // Paint cross
            if let Some(cross_galley) = cross_galley {
                let cross_rect = cross_galley.rect;
                let oy = toast.height / 2. - cross_height / 2.;
                let ox = toast.width - cross_width - cross_x_padding.1 - padding.x;
                let cross_pos = rect.min + vec2(ox, oy);
                painter.galley(cross_pos, cross_galley);

                let screen_cross = Rect {
                    max: cross_pos + cross_rect.max.to_vec2(),
                    min: cross_pos,
                };

                if let Some(pos) = ctx.input(|inp| inp.pointer.press_origin()) {
                    if screen_cross.contains(pos) && !*held {
                        dismiss = Some(i);
                        *held = true;
                    }
                }
            }

            // Draw duration
            if let Some((initial, current)) = toast.duration {
                if !toast.state.disappearing() {
                    painter.line_segment(
                        [
                            rect.min + vec2(0., toast.height),
                            rect.max - vec2((1. - (current / initial)) * toast.width, 0.),
                        ],
                        Stroke::new(4., fg_color),
                    );
                }
            }

            toast.adjust_next_pos(&mut pos, *anchor, *spacing);

            // Animations
            if toast.state.appearing() {
                update = true;
                toast.value += ctx.input(|inp| inp.stable_dt * (*speed));

                if toast.value >= 1. {
                    toast.value = 1.;
                    toast.state = ToastState::Idle;
                }
            } else if toast.state.disappearing() {
                update = true;
                toast.value -= ctx.input(|inp| inp.stable_dt * (*speed));

                if toast.value <= 0. {
                    toast.state = ToastState::Disappeared;
                }
            }
        }

        if update {
            ctx.request_repaint();
        }

        if let Some(i) = dismiss {
            self.toasts[i].dismiss();
        }
    }
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

fn ease_in_cubic(x: f32) -> f32 {
    1. - (1. - x).powi(3)
}

#[macro_export]
macro_rules! toast_error {
    ($($args:tt)*) => { $crate::TOAST_MSG.lock().error(format!($($args)*)) };
}

#[macro_export]
macro_rules! toast_warn {
    ($($args:tt)*) => { $crate::TOAST_MSG.lock().warning(format!($($args)*)) };
}

#[macro_export]
macro_rules! toast_info {
    ($($args:tt)*) => { $crate::TOAST_MSG.lock().info(format!($($args)*)) };
}

#[macro_export]
macro_rules! toast_success {
    ($($args:tt)*) => { $crate::TOAST_MSG.lock().success(format!($($args)*)) };
}
