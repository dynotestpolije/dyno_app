use std::{
    rc::Rc,
    sync::atomic::{AtomicU8, Ordering},
};

use super::button::ButtonKind;
use crate::assets::{ICO_ERROR, ICO_INFO, ICO_WARN};
use eframe::egui::{Color32, ColorImage, TextureHandle, Vec2};

use eframe::NativeOptions;

#[derive(Default, Debug, Clone, Copy)]
pub enum PopupLevel {
    #[default]
    Info,
    Warning,
    Error,
}
impl From<PopupLevel> for rfd::MessageLevel {
    #[inline]
    fn from(value: PopupLevel) -> Self {
        match value {
            PopupLevel::Info => Self::Info,
            PopupLevel::Warning => Self::Warning,
            PopupLevel::Error => Self::Error,
        }
    }
}
impl From<PopupLevel> for Color32 {
    fn from(value: PopupLevel) -> Self {
        match value {
            PopupLevel::Info => Color32::GREEN,
            PopupLevel::Warning => Color32::YELLOW,
            PopupLevel::Error => Color32::RED,
        }
    }
}

pub struct PopupWindow<const BTN_SIZE: usize = 1> {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) level: PopupLevel,

    pub(crate) buttons: [(ButtonKind, &'static str); BTN_SIZE],
    pub(crate) button_selected: Rc<AtomicU8>,

    pub(crate) icon_texture: Option<TextureHandle>,
    pub(crate) opened: bool,
}

impl<const BTN_SIZE: usize> PopupWindow<BTN_SIZE> {
    pub fn new<S>(title: S, description: S, buttons: [(ButtonKind, &'static str); BTN_SIZE]) -> Self
    where
        S: ToString,
    {
        Self {
            buttons,
            title: title.to_string(),
            description: description.to_string(),
            level: Default::default(),
            opened: true,
            button_selected: Default::default(),
            icon_texture: None,
        }
    }

    pub fn show(mut self) -> ButtonKind {
        let option_icon = match self.level {
            PopupLevel::Info => ICO_INFO.clone(),
            PopupLevel::Warning => ICO_WARN.clone(),
            PopupLevel::Error => ICO_ERROR.clone(),
        };
        let native_options = NativeOptions {
            always_on_top: true,
            resizable: false,
            initial_window_size: Some(Vec2 { x: 300.0, y: 300.0 }),
            min_window_size: Some(Vec2 { x: 300.0, y: 300.0 }),
            icon_data: option_icon.as_ref().map(|(i, _)| i.clone()),
            ..Default::default()
        };
        let btn_selected = self.button_selected.clone();
        let title = self.title.clone();

        eframe::run_native(
            title.as_ref(),
            native_options,
            Box::new(move |ctx| {
                // eframe::egui::TextureHandle::

                self.icon_texture = option_icon.map(|(ico, size)| {
                    let img =
                        ColorImage::from_rgba_unmultiplied([size[0] as _, size[1] as _], &ico.rgba);
                    ctx.egui_ctx
                        .load_texture("icon_popup", img, Default::default())
                });
                Box::new(self)
            }),
        )
        .ok();

        btn_selected.load(Ordering::SeqCst).into()
    }
}

impl<const BTN_SIZE: usize> PopupWindow<BTN_SIZE> {
    pub fn set_level(mut self, level: PopupLevel) -> Self {
        self.level = level;
        self
    }
}

impl<const BTN_SIZE: usize> eframe::App for PopupWindow<BTN_SIZE> {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        use eframe::egui::*;
        if !self.opened {
            frame.close()
        }
        let (header, desc) = match self.level {
            PopupLevel::Info => (
                RichText::new("INFO!").color(self.level),
                RichText::new(&self.description),
            ),
            PopupLevel::Warning => (
                RichText::new("WARNING!").color(self.level),
                RichText::new(format!(
                    "Something Went Wrong!\n{}",
                    &self.description.clone()
                )),
            ),
            PopupLevel::Error => (
                RichText::new("ERROR!").color(self.level),
                RichText::new(format!(
                    "Something Went Wrong!\n{}",
                    &self.description.clone()
                )),
            ),
        };

        let horz_top = |hui: &mut Ui| {
            if let Some(texture) = &self.icon_texture {
                hui.image(texture.id(), texture.size_vec2());
            }
            hui.heading(header.clone());
            hui.label(desc.clone());
        };

        Window::new(&self.title)
            .open(&mut self.opened)
            .title_bar(true)
            .show(ctx, |vui| {
                vui.vertical_centered_justified(|vvui| {
                    vvui.horizontal_top(horz_top);
                    vvui.separator();
                    vvui.add(Label::new(&self.description));
                    vvui.separator();
                    for (btn, name) in self.buttons {
                        if vvui
                            .add(Button::new(btn.to_string()).fill(self.level))
                            .on_hover_text(name)
                            .clicked()
                        {
                            self.button_selected.store(btn as u8, Ordering::SeqCst);
                        }
                    }
                });
            });

        if !self.opened
            || !matches!(
                self.button_selected.load(Ordering::SeqCst).into(),
                ButtonKind::Any
            )
        {
            frame.close()
        }
    }
}

impl PopupWindow<2> {
    pub fn info<S>(title: S, desc: S) -> ButtonKind
    where
        S: ToString,
    {
        let btns = [
            (ButtonKind::Ok, "click 'OK' to accuire proces"),
            (ButtonKind::Cancel, "click 'Cancel', to cancel the proces"),
        ];
        Self::new(title, desc, btns)
            .set_level(PopupLevel::Info)
            .show()
    }
    pub fn warn<S>(title: S, desc: S) -> ButtonKind
    where
        S: ToString,
    {
        let btns = [
            (ButtonKind::Ok, "click 'OK' to accuire proces"),
            (ButtonKind::Cancel, "click 'Cancel', to cancel the proces"),
        ];
        Self::new(title, desc, btns)
            .set_level(PopupLevel::Warning)
            .show()
    }
}
impl PopupWindow<3> {
    pub fn err<S>(title: S, desc: S) -> ButtonKind
    where
        S: ToString,
    {
        let btns = [
            (ButtonKind::Ok, "Click 'OK' to continue the process"),
            (ButtonKind::Cancel, "Click 'Cancel' to cancel the process"),
            (ButtonKind::Close, "Click 'Close' to quit/close the process"),
        ];
        Self::new(title, desc, btns)
            .set_level(PopupLevel::Error)
            .show()
    }
}

#[test]
fn popup_window() {
    match PopupWindow::info("Hello", "Test Popup Hello") {
        ButtonKind::Ok => eprintln!("Button Ok Clicked"),
        ButtonKind::Cancel => eprintln!("Button Calcel Clicked"),
        unknown => eprintln!("Uknown '{unknown}' Button Clicked"),
    }
}
