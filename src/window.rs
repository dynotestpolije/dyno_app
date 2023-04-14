#![allow(unused)]

use crate::{widgets::DynoWidgets, PACKAGE_INFO};
use eframe::egui::{Align2, Context, Vec2, Window};

#[inline]
pub fn show_about(ctx: &Context, open: &mut bool) {
    let package_info = &PACKAGE_INFO;
    Window::new("About")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.heading(package_info.name);
            ui.label(format!("Version {}", package_info.version));

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

            ui.separator();

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

                // (!) Rust incremental compilation bug:
                // When the 'license' field is changed in the crate's Cargo.toml,
                // source files that include that field through `env!()` macros
                // are not picked up for recompilation.
                // Always do `cargo clean` + full rebuild when changing Cargo.toml metadata.
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
        });
}

pub fn show_config(ctx: &Context, open: &mut bool) {
    todo!()
}

pub fn show_help(ctx: &Context, open: &mut bool) {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmOption {
    Def,
    Yes,
    No,
}

#[inline(always)]
pub fn confirm_quit(ctx: &Context, open: &mut bool) -> ConfirmOption {
    let mut ret_value = ConfirmOption::Def;
    let painter = ctx.layer_painter(eframe::egui::LayerId::new(
        eframe::egui::Order::Background,
        eframe::egui::Id::new("confirmation_quit"),
    ));
    painter.rect_filled(
        ctx.input(|inp| inp.screen_rect()),
        0.0,
        eframe::egui::Color32::from_black_alpha(192),
    );
    let horiz = |ui: &mut eframe::egui::Ui| {
        ui.horizontal(|ui| {
            if ui.button("Yes").clicked() {
                ret_value = ConfirmOption::Yes;
            }
            if ui.button("No").clicked() {
                ret_value = ConfirmOption::No;
            }
        })
    };

    eframe::egui::Window::new("Do you want to quit?")
        .anchor(Align2::CENTER_CENTER, [-50f32, 0f32])
        .open(open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, horiz);

    ret_value
}
