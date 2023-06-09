use crate::{widgets::DynoWidgets, PACKAGE_INFO};
use eframe::egui::Window;
use eframe::epaint::Vec2;

use dyno_core::serde;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(crate = "serde")]
pub struct AboutWindow {
    open: bool,
}

impl AboutWindow {
    pub fn new() -> Self {
        Self { open: false }
    }
}

impl super::WindowState for AboutWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        _control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
    ) {
        let uidraw_collapsing = |ui: &mut eframe::egui::Ui| {
            ui.horizontal(|ui| {
                for (author_name, author_email) in PACKAGE_INFO.authors() {
                    if let Some(author_email) = author_email {
                        if !["noreply@", "no-reply@", "@users.noreply."]
                            .iter()
                            .any(|no_reply| author_email.contains(no_reply))
                        {
                            ui.hyperlink_with_icon_to(
                                author_name,
                                format!("mailto:{author_email:}"),
                            );
                        } else {
                            ui.label(format!("\u{1F464} {author_name:}"));
                        }
                    } else {
                        ui.label(format!("\u{1F464} {author_name:}"));
                    }
                }
            });

            if let Some(license) = PACKAGE_INFO.license {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                    ui.label("License: ");

                    license.split_whitespace().for_each(|s| match s {
                        operator @ ("OR" | "AND" | "WITH") => {
                            ui.label(format!(" {} ", operator.to_lowercase()));
                        }
                        license => {
                            ui.hyperlink_with_icon_to(
                                license,
                                format!("https://spdx.org/licenses/{license:}.html"),
                            );
                        }
                    });
                });
            };

            if let Some(license_file) = PACKAGE_INFO.license_file {
                ui.separator();
                ui.label(format!(
                    "License: See the {license_file:} file for details."
                ));
            };
        };

        Window::new("About")
            .open(&mut self.open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui: &mut eframe::egui::Ui| {
                ui.heading(PACKAGE_INFO.name);
                ui.label(format!("Version {}", PACKAGE_INFO.version));

                ui.separator();

                if let Some(description) = PACKAGE_INFO.description {
                    ui.label(description);
                    ui.separator();
                }

                ui.horizontal(|ui| {
                    if let Some(homepage) = PACKAGE_INFO.homepage {
                        ui.hyperlink_with_icon_to("Home page", homepage);
                    }
                    if let Some(repository) = PACKAGE_INFO.repository {
                        ui.hyperlink_with_icon_to("Repository", repository);
                    }
                });

                ui.separator();

                ui.collapsing("Authors", uidraw_collapsing);
            });
    }

    #[inline]
    fn set_open(&mut self, open: bool) {
        self.open = open
    }
    #[inline]
    fn is_open(&self) -> bool {
        self.open
    }
}
