use crate::widgets::{DisplayStyle, DisplayStylePreset};
use dyno_core::{derive_more::Display, paste::paste};
use eframe::egui::{
    vec2, Align2, FontFamily, FontId, Key, Rect, Response, Sense, Stroke, Ui, Widget,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum ButtonKind {
    #[display(fmt = "\u{2714} Ok")]
    Ok,

    #[display(fmt = "\u{1F6AB} Cancel")]
    Cancel,

    #[display(fmt = "\u{2714} Apply")]
    Apply,

    #[display(fmt = "\u{1F504} Reset")]
    Reset,

    #[display(fmt = "\u{1F5C1} Open")]
    Open,

    #[display(fmt = "\u{1F4BE} Save")]
    Save,

    #[display(fmt = "\u{1F4BE} Save As...")]
    SaveAs,

    #[display(fmt = "\u{1F5D9} Close")]
    Close,

    #[display(fmt = "\u{1F5D1} Delete")]
    Delete,

    #[display(fmt = "\u{25B6} Play")]
    Play,

    #[display(fmt = "\u{23F8} Pause")]
    Pause,

    #[display(fmt = "\u{23F9} Stop")]
    Stop,

    #[display(fmt = "\u{23FA} Record")]
    Record,

    #[display(fmt = "\u{23ED} Next")]
    Next,

    #[display(fmt = "\u{23EE} Previous")]
    Previous,

    #[display(fmt = "\u{26F6} Full Screen")]
    FullScreen,

    #[display(fmt = "\u{1F3B2} Random")]
    Random,

    #[display(fmt = "\u{270F} Edit")]
    Edit,

    #[display(fmt = "\u{2605} Favorite")]
    Favorite,

    #[display(fmt = "\u{2606} Unfavorite")]
    Unfavorite,

    #[display(fmt = "\u{1F507} Mute")]
    Mute,

    #[display(fmt = "\u{1F50A} Unmute")]
    Unmute,

    #[display(fmt = "\u{1F512} Lock")]
    Lock,

    #[display(fmt = "\u{1F513} Unlock")]
    Unlock,

    #[display(fmt = "\u{1F503} Refresh")]
    Refresh,

    #[display(fmt = "\u{1F5CB} New")]
    New,

    #[display(fmt = "\u{1F5D0} Copy")]
    Copy,

    #[display(fmt = "\u{1F4CB} Paste")]
    Paste,

    #[display(fmt = "\u{2702} Cut")]
    Cut,

    #[display(fmt = "\u{2718} No")]
    No,

    #[display(fmt = "")]
    Any,
}

impl ButtonKind {
    #[inline(always)]
    pub fn name_button_popup(self, desc: &'_ str) -> String {
        format!("clickk '{self}' to {desc}")
    }
}
impl From<u8> for ButtonKind {
    fn from(b: u8) -> Self {
        match b {
            0 => Self::Ok,
            1 => Self::Cancel,
            2 => Self::Apply,
            3 => Self::Reset,
            4 => Self::Open,
            5 => Self::Save,
            6 => Self::SaveAs,
            7 => Self::Close,
            8 => Self::Delete,
            9 => Self::Play,
            10 => Self::Pause,
            11 => Self::Stop,
            12 => Self::Record,
            13 => Self::Next,
            14 => Self::Previous,
            15 => Self::FullScreen,
            16 => Self::Random,
            17 => Self::Edit,
            18 => Self::Favorite,
            19 => Self::Unfavorite,
            20 => Self::Mute,
            21 => Self::Unmute,
            22 => Self::Lock,
            23 => Self::Unlock,
            24 => Self::Refresh,
            25 => Self::New,
            26 => Self::Copy,
            27 => Self::Paste,
            28 => Self::Cut,
            29 => Self::No,
            _ => Self::Any,
        }
    }
}

impl Default for ButtonKind {
    fn default() -> Self {
        Self::Any
    }
}

macro_rules!  standart_button {
    ($traits:ident {$( $name: ident),*}) => {
        pub trait $traits {
            fn button_ext(&mut self, button_kind: impl ToString) -> Response;
            fn small_button_ext(&mut self, button_kind: impl ToString) -> Response;
        paste!($(
            #[allow(unused)]
            #[inline(always)]
            fn [<$name:lower _button>](&mut self) -> Response {
                self.button_ext(ButtonKind::$name)
            }
            #[allow(unused)]
            #[inline(always)]
            fn [<small_ $name:lower _button>](&mut self) -> Response {
                self.small_button_ext(ButtonKind::$name)
            }
        )*);
        }
    };
}

standart_button!(ButtonExt {
    Ok,
    Cancel,
    Apply,
    Reset,
    Open,
    Save,
    SaveAs,
    Close,
    Delete,
    Play,
    Pause,
    Stop,
    Record,
    Next,
    Previous,
    FullScreen,
    Random,
    Edit,
    Favorite,
    Unfavorite,
    Mute,
    Unmute,
    Lock,
    Unlock,
    Refresh,
    New,
    Copy,
    Paste,
    Cut,
    No,
    Any
});

impl ButtonExt for eframe::egui::Ui {
    #[allow(unused)]
    #[inline(always)]
    fn small_button_ext(&mut self, button_kind: impl ToString) -> Response {
        self.small_button(button_kind.to_string())
    }
    #[allow(unused)]
    #[inline(always)]
    fn button_ext(&mut self, button_kind: impl ToString) -> Response {
        self.button(button_kind.to_string())
    }
}

// ----------------------------------------------------------------------------

/// Combined into one function (rather than two) to make it easier
/// for the borrow checker.
type GetSetValue<'a> = Box<dyn 'a + FnMut(Option<bool>) -> bool>;

fn get(get_set_value: &mut GetSetValue<'_>) -> bool {
    (get_set_value)(None)
}

fn set(get_set_value: &mut GetSetValue<'_>, value: bool) {
    (get_set_value)(Some(value));
}

// ----------------------------------------------------------------------------

#[non_exhaustive]
#[derive(Clone, Copy, Display, Eq, PartialEq)]
pub enum ButtonIndicatorBehavior {
    #[display(fmt = "Toggle")]
    Toggle,

    #[display(fmt = "Hold")]
    Hold,
}

// ----------------------------------------------------------------------------

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct ButtonIndicator<'a> {
    get_set_value: GetSetValue<'a>,
    width: f32,
    height: f32,
    label: Option<String>,
    style: DisplayStyle,
    animated: bool,
    interactive: bool,
    margin: f32,
    behavior: ButtonIndicatorBehavior,
}

impl<'a> ButtonIndicator<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self::from_get_set(move |v: Option<bool>| {
            if let Some(v) = v {
                *value = v;
            }
            *value
        })
    }

    pub fn toggle(value: &'a mut bool) -> Self {
        Self::new(value).behavior(ButtonIndicatorBehavior::Toggle)
    }

    pub fn hold(value: &'a mut bool) -> Self {
        Self::new(value).behavior(ButtonIndicatorBehavior::Hold)
    }

    pub fn from_get_set(get_set_value: impl 'a + FnMut(Option<bool>) -> bool) -> Self {
        Self {
            get_set_value: Box::new(get_set_value),
            width: 64.0,
            height: 40.0,
            label: None,
            style: DisplayStylePreset::Default.style(),
            animated: true,
            interactive: true,
            margin: 0.2,
            behavior: ButtonIndicatorBehavior::Toggle,
        }
    }

    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<f32>) -> Self {
        self.height = height.into();
        self
    }

    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn style(mut self, style: DisplayStyle) -> Self {
        self.style = style;
        self
    }

    pub fn style_preset(mut self, preset: DisplayStylePreset) -> Self {
        self.style = preset.style();
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn margin(mut self, margin: impl Into<f32>) -> Self {
        self.margin = margin.into();
        self
    }

    pub fn behavior(mut self, behavior: ButtonIndicatorBehavior) -> Self {
        self.behavior = behavior;
        self
    }
}

impl<'a> Widget for ButtonIndicator<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let desired_size = vec2(self.width, self.height);

        let (rect, mut response) = ui.allocate_exact_size(
            desired_size,
            if self.interactive {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            },
        );

        match self.behavior {
            ButtonIndicatorBehavior::Toggle => {
                if response.clicked() {
                    let value = get(&mut self.get_set_value);
                    set(&mut self.get_set_value, !value);

                    response.mark_changed();
                }
            }
            ButtonIndicatorBehavior::Hold => {
                if response.drag_started() || response.drag_released() {
                    set(&mut self.get_set_value, response.dragged());
                    response.mark_changed();
                }

                if response.has_focus() {
                    ui.input(|input| {
                        if input.key_pressed(Key::Enter) || input.key_pressed(Key::Space) {
                            set(&mut self.get_set_value, true);
                            response.mark_changed();
                        }

                        if input.key_released(Key::Enter) || input.key_released(Key::Space) {
                            set(&mut self.get_set_value, false);
                            response.mark_changed();
                        }
                    });
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let visuals = *ui.style().interact(&response);

            let value = if self.animated {
                ui.ctx()
                    .animate_bool(response.id, get(&mut self.get_set_value))
            } else {
                #[allow(clippy::collapsible_else_if)]
                if get(&mut self.get_set_value) {
                    1.0
                } else {
                    0.0
                }
            };

            ui.painter()
                .rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

            let top_rect = Rect::from_min_max(rect.left_top(), rect.right_center());
            let bottom_rect = Rect::from_min_max(rect.left_center(), rect.right_bottom());

            let margin = (self.height / 2.0) * self.margin;

            {
                let indicator_rect = if self.label.is_some() { top_rect } else { rect };

                ui.painter().rect(
                    indicator_rect.shrink(margin),
                    4.0,
                    self.style.background_color,
                    Stroke::NONE,
                );

                ui.painter().rect(
                    indicator_rect.shrink(margin + 2.0),
                    4.0,
                    self.style.foreground_color_blend(value),
                    Stroke::NONE,
                );
            }

            if let Some(label) = self.label {
                ui.painter().text(
                    bottom_rect.center() - vec2(0.0, margin / 2.0),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::new(bottom_rect.height() - margin, FontFamily::Proportional),
                    visuals.text_color(),
                );
            }
        }

        response
    }
}
