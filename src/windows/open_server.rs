use std::ops::Index;

use dyno_core::dynotests::DynoTest;
use eframe::egui::{Button, Id, LayerId, Layout, RichText, Ui, Window};
use eframe::emath::{Align, Align2};
use eframe::epaint::{vec2, Color32, Rounding, Vec2};
use egui_extras::{Column, TableBuilder};

use crate::widgets::button::ButtonExt;
use crate::{toast_error, toast_warn};

#[derive(Debug, Clone, Default)]
pub struct OpenServerWindow {
    data: Vec<DynoTest>,
    open: bool,
}
impl OpenServerWindow {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_data(&mut self, data: Vec<DynoTest>) {
        self.data = data;
    }
}

impl super::WindowState for OpenServerWindow {
    fn show_window(
        &mut self,
        ctx: &eframe::egui::Context,
        control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
    ) {
        ctx.layer_painter(LayerId::new(
            eframe::egui::Order::Background,
            Id::new("confirmation_popup_unsaved"),
        ))
        .rect_filled(
            ctx.input(|inp| inp.screen_rect()),
            0.0,
            Color32::from_black_alpha(192),
        );

        let ui_window = |ui: &mut Ui| {
            ui.heading("List File in Server");
            ui.add_space(10.);
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(Layout::left_to_right(Align::Center))
                .columns(Column::remainder().at_least(40.0), 6)
                .resizable(true)
                .header(20.0, |mut head_ui| {
                    head_ui.col(|col_ui| {
                        col_ui.strong("id");
                    });
                    head_ui.col(|col_ui| {
                        col_ui.strong("info id");
                    });
                    head_ui.col(|col_ui| {
                        col_ui.strong("verified");
                    });
                    head_ui.col(|col_ui| {
                        col_ui.strong("updated");
                    });
                    head_ui.col(|col_ui| {
                        col_ui.strong("created");
                    });
                    head_ui.col(|col_ui| {
                        col_ui.strong("open");
                    });
                })
                .body(|body_ui| {
                    let row_height = 18.0;
                    let num_rows = self.data.len();
                    body_ui.rows(row_height, num_rows, |row_idx, mut row| {
                        let DynoTest {
                            id,
                            info_id,
                            verified,
                            updated_at,
                            created_at,
                            data_url,
                        data_checksum,
                            ..
                        } = self.data.index(row_idx);

                        row.col(|ui| {
                            ui.label(id.to_string());
                        });
                        row.col(|ui| {
                            ui.label((*info_id).map(|x| x.to_string()).unwrap_or_default());
                        });
                        row.col(|ui| {
                            ui.label(if *verified {
                                "verified"
                            } else {
                                "not verified"
                            });
                        });
                        row.col(|ui| {
                            ui.label(updated_at.format("%d-%m-%Y %T").to_string());
                        });
                        row.col(|ui| {
                            ui.label(created_at.format("%d-%m-%Y %T").to_string());
                        });
                        row.col(|ui| {
                            let open_btn = ui.small_open_button();
                            match (open_btn.clicked(), control.service.api()) {
                                (true, None) => {
                                    toast_error!("Something Wrong, Api is not Connected! trying to reconnecting..");
                                    control.service.reconnect_api(&control.config);
                                }
                                (true, Some(api)) => {
                                    control.set_loading();
                                    api.load_dyno_file(data_url.clone(), data_checksum.clone(), control.service.tx());
                                }
                                _ => {}
                            }
                        });
                    })
                });

            ui.add_space(10.);
            let refresh_btn = ui.add(
                Button::new(RichText::new("Refresh").color(Color32::BLACK))
                    .rounding(Rounding::same(4.))
                    .fill(Color32::LIGHT_BLUE)
                    .min_size(vec2(280., 30.)),
            );
            ui.add_space(20.);
            if refresh_btn.clicked() {
                match control.service.api() {
                    Some(api) => api.get_dyno(control.service.tx()),
                    None => {
                        toast_warn!("Not connected to API, trying to reconnecting..");
                        control.service.reconnect_api(&control.config);
                    }
                }
            }
        };

        Window::new("Save DynoTests to Server")
            .id("dyno_save_server".into())
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .open(&mut self.open)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| ui.vertical_centered_justified(ui_window));
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
