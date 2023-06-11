use crate::{toast_error, toast_warn, widgets::DynoWidgets};
use dyno_core::{
    role::Roles,
    users::{UserLogin, UserRegistration},
    validate_email, validate_nim, validate_password, PasswordStrength,
};
use eframe::egui::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthSection {
    Login,
    Register,
}

impl Default for AuthSection {
    fn default() -> Self {
        Self::Login
    }
}

impl AuthSection {
    fn as_str(&self) -> &'static str {
        match self {
            AuthSection::Login => "Login",
            AuthSection::Register => "Register",
        }
    }
    fn opposite(self) -> Self {
        match self {
            AuthSection::Login => AuthSection::Register,
            AuthSection::Register => AuthSection::Login,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct LoginSection {
    data: UserLogin,
    status_nim: Option<Color32>,
    status_password: Option<Color32>,
    show_password: bool,
    password_strength: PasswordStrength,
}
impl LoginSection {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            data: UserLogin { nim, password },
            status_nim,
            status_password,
            show_password,
            password_strength,
        } = self;

        text_edit_validate(ui, nim, status_nim, "nim: ", validate_nim);
        text_edit_validate_password(
            "password: ",
            ui,
            password,
            status_password,
            password_strength,
            !*show_password,
            validate_password,
        );
        ui.horizontal(|ui| {
            ui.checkbox(show_password, "show password");
            let (val, col) = password_strength.percent_color();
            ui.add(
                ProgressBar::new(val)
                    .fill(Color32::from_rgb(col[0], col[1], col[2]))
                    .text(password_strength.desc()),
            );
        });
    }
}

#[derive(Debug, Default, Clone)]
pub struct RegistrationSection {
    data: UserRegistration,
    status_nim: Option<Color32>,
    status_email: Option<Color32>,
    status_password: Option<Color32>,
    status_confirm_password: Option<Color32>,
    password_strength: PasswordStrength,
    show_password: bool,
}
impl RegistrationSection {
    fn ui(&mut self, ui: &mut Ui) {
        let Self {
            data:
                UserRegistration {
                    nim,
                    email,
                    password,
                    confirm_password,
                    role,
                },
            status_nim,
            status_email,
            status_password,
            status_confirm_password,
            password_strength,
            show_password,
        } = self;
        text_edit_validate(ui, nim, status_nim, "nim: ", validate_nim);
        text_edit_validate(ui, email, status_email, "email: ", validate_email);
        text_edit_validate_password(
            "password: ",
            ui,
            password,
            status_password,
            password_strength,
            !*show_password,
            validate_password,
        );
        text_edit_validate_password(
            "confirm password: ",
            ui,
            confirm_password,
            status_confirm_password,
            &mut password_strength.clone(),
            !*show_password,
            |s| {
                if s != password {
                    Err(crate::DynoErr::validation_error(
                        "Password not matching with first password",
                    ))
                } else {
                    Ok(())
                }
            },
        );

        ui.combobox_from_slice("Role", role, &[Roles::User, Roles::Guest]);

        ui.horizontal(|ui| {
            ui.checkbox(show_password, "show password");
            let (val, col) = password_strength.percent_color();
            ui.add(
                ProgressBar::new(val)
                    .fill(Color32::from_rgb(col[0], col[1], col[2]))
                    .text(password_strength.desc()),
            );
        });
    }
}

#[derive(Debug, Default, Clone)]
pub struct AuthWindow {
    open: bool,
    section: AuthSection,
    login: LoginSection,
    register: RegistrationSection,
}

impl AuthWindow {
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::WindowState for AuthWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
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

        ctx.input(|i| {
            if i.key_down(Key::Escape) {
                self.open = false;
            }
        });

        Window::new(self.section.as_str())
            .id("dyno_auth_window".into())
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .open(&mut self.open)
            .fixed_size(vec2(300., 700.))
            .default_size(vec2(300., 700.))
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|vertui| {
                    vertui.add_space(10.);
                    let section_opp = self.section.opposite();
                    let btn_section = vertui.add(
                        Button::new(RichText::new(section_opp.as_str()).color(Color32::BLACK))
                            .rounding(Rounding::same(4.))
                            .fill(Color32::LIGHT_BLUE),
                    );
                    if btn_section.clicked() {
                        self.section = section_opp;
                    }

                    vertui.add_space(30.);
                    match self.section {
                        AuthSection::Login => self.login.ui(vertui),
                        AuthSection::Register => self.register.ui(vertui),
                    }

                    vertui.add_space(30.);
                    let submit_btn = vertui.add(
                        Button::new(RichText::new("Submit").color(Color32::BLACK))
                            .rounding(Rounding::same(4.))
                            .fill(Color32::LIGHT_BLUE)
                            .min_size(vec2(280., 30.)),
                    );

                    if submit_btn.clicked() {
                        match control.api() {
                            Some(api) => api.login(self.login.data.clone(), control.tx().clone()),
                            None => {
                                toast_warn!("Aplication not connected Api Server - trying reconnecting.. and try again.");
                                if let Err(err) = control.reconnect_api() {
                                    toast_error!("Failed to reconnect to Api - {err}");
                                }
                            }
                        }
                    }
                    vertui.add_space(10.);
                });
            });
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

#[inline]
fn text_edit_validate<'a>(
    ui: &mut Ui,
    input: &'a mut String,
    status: &mut Option<Color32>,
    hint: &'static str,
    validation_fn: impl FnOnce(&'a str) -> crate::DynoResult<()>,
) -> Response {
    let resp = ui.add(
        TextEdit::singleline(input)
            .text_color_opt(*status)
            .hint_text(hint)
            .margin(vec2(10., 10.)),
    );
    if !input.is_empty() {
        if let Some(status_text) = match validation_fn(input) {
            Ok(()) => None,
            Err(err) => Some(Label::new(RichText::new(err.desc).color(Color32::RED))),
        } {
            ui.add(status_text);
            *status = Some(Color32::RED);
        } else {
            ui.add_space(4.);
            *status = None;
        }
    } else {
        ui.add_space(4.)
    }
    resp
}

#[inline]
fn text_edit_validate_password<'a>(
    hint: &'static str,
    ui: &mut Ui,
    input: &'a mut String,
    status: &mut Option<Color32>,
    strength: &mut PasswordStrength,
    password: bool,
    validation_fn: impl FnOnce(&'a str) -> crate::DynoResult<()>,
) -> Response {
    let resp = ui.add(
        TextEdit::singleline(input)
            .text_color_opt(*status)
            .password(password)
            .hint_text(hint)
            .margin(vec2(10., 10.)),
    );
    if !input.is_empty() {
        let is_valid = if let Some(status_text) = match validation_fn(input) {
            Ok(()) => None,
            Err(err) => Some(Label::new(RichText::new(err.desc).color(Color32::RED))),
        } {
            ui.add(status_text);
            false
        } else {
            ui.add_space(4.);
            true
        };
        if resp.changed() {
            *strength = PasswordStrength::new(&input);
            *status = dyno_core::ternary!((is_valid)?(None): (Some(Color32::RED)));
        }
    } else {
        ui.add_space(4.)
    }

    resp
}
