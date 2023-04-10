use dynotest_app::widgets::DynoWidgets;
use serde::{Deserialize, Serialize};

use crate::{controller::Controller, PACKAGE_INFO};
use eframe::egui::*;

#[derive(Deserialize, Serialize)]
pub struct StartupWindow {
    open: bool,
}

impl StartupWindow {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Box<dyn eframe::App + 'static> {
        Box::new(Self { open: true })
    }
}

impl eframe::App for StartupWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(2));
        eframe::egui::Window::new("StartupWindow")
            .auto_sized()
            .open(&mut self.open)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                let package_info = &PACKAGE_INFO;
                ui.heading(format!(
                    "Welcome to {} - version {}",
                    package_info.app_name, package_info.version
                ));
                ui.separator();
                if let Some(description) = package_info.description {
                    ui.label(description);
                    ui.separator();
                }
                ui.horizontal(|ui| {
                    if let Some(homepage) = package_info.homepage {
                        ui.hyperlink_with_icon_to("Home page", homepage);
                    }

                    if let Some(repository) = package_info.repository {
                        ui.hyperlink_with_icon_to("Repository", repository);
                    }
                });

                ui.collapsing("Authors", |ui| {
                    ui.horizontal(|ui| {
                        for (author_name, author_email) in package_info.authors() {
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

                    if let Some(license) = package_info.license {
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

                    if let Some(license_file) = package_info.license_file {
                        ui.separator();
                        ui.label(format!(
                            "License: See the {license_file:} file for details."
                        ));
                    };
                });

                ui.separator();
            });

        if !self.open {
            frame.close()
        }
    }
}
