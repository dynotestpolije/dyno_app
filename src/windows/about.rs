use eframe::egui::Window;
use eframe::epaint::Vec2;

use crate::widgets::DynoWidgets;
use crate::PACKAGE_INFO;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AboutWindow {
    open: bool,
}
impl AboutWindow {
    pub fn new() -> Self {
        AboutWindow::default()
    }
}

impl super::WindowState for AboutWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
        state: &mut crate::state::DynoState,
    ) {
        let open_about_window = state.show_about();
        if open_about_window {
            return;
        }
        self.open = open_about_window;
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

        let uidraw = |ui: &mut eframe::egui::Ui| {
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
        };

        Window::new("About")
            .open(&mut self.open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, uidraw);

        state.set_show_about(self.open);
    }
}
