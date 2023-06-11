use crate::widgets::button::ButtonExt;
use dyno_core::serde;
use eframe::egui::{
    Align2, Color32, Context, Id, InnerResponse, Key, LayerId, Order, Ui, Vec2, Window,
};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(crate = "serde")]
pub struct ConfirmQuitWindow {
    open: bool,
}
impl ConfirmQuitWindow {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::WindowState for ConfirmQuitWindow {
    fn show_window(
        &mut self,
        ctx: &Context,
        _control: &mut crate::control::DynoControl,
        state: &mut crate::state::DynoState,
    ) {
        ctx.layer_painter(LayerId::new(
            Order::Background,
            Id::new("confirmation_popup_unsaved"),
        ))
        .rect_filled(
            ctx.input(|inp| inp.screen_rect()),
            0.0,
            Color32::from_black_alpha(192),
        );

        if let Some(InnerResponse {
            inner: Some(Some(b)),
            ..
        }) = Window::new("Do you wanna close the Application?")
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui: &mut Ui| {
                ui.small("click 'Ok' to close the app or 'No' to rebort close event");
                ui.horizontal(|horz_ui| {
                    if horz_ui.ok_button().clicked() || horz_ui.input(|i| i.key_down(Key::Enter)) {
                        Some(true)
                    } else if horz_ui.no_button().clicked()
                        || horz_ui.input(|i| i.key_down(Key::Escape))
                    {
                        Some(false)
                    } else {
                        None
                    }
                })
                .inner
            })
        {
            state.set_quitable(b);
            state.set_quit(b);
        }
    }

    #[inline]
    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.open
    }
}
