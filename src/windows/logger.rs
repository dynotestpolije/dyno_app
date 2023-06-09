use std::{env, fs, path::PathBuf};

use eframe::{
    egui::{widgets, Align, Context, Layout, ScrollArea, Ui, Window},
    epaint::Color32,
};
use itertools::Itertools;

use crate::widgets::{button::ButtonExt, DynoFileManager, DynoWidgets};
use dyno_core::{log::Level, RECORDS_LOGGER};

const SIZE_LEVEL: usize = Level::Trace as usize;
const LEVELS: [Level; SIZE_LEVEL] = [
    Level::Error,
    Level::Warn,
    Level::Info,
    Level::Debug,
    Level::Trace,
];

#[inline(always)]
pub const fn level_color(lvl: Level) -> Color32 {
    match lvl {
        Level::Error => Color32::RED,
        Level::Warn => Color32::YELLOW,
        Level::Info => Color32::GREEN,
        Level::Debug => Color32::LIGHT_GRAY,
        Level::Trace => Color32::WHITE,
    }
}

#[derive(Debug, Clone)]
pub struct LoggerWindow {
    open: bool,
    loglevels: [bool; SIZE_LEVEL],
    term: String,
    case_sensitive: bool,
    max_log_len: usize,
}

impl Default for LoggerWindow {
    fn default() -> Self {
        Self {
            open: false,
            loglevels: [true, true, true, false, false],
            term: String::with_capacity(128),
            case_sensitive: false,
            max_log_len: 1000,
        }
    }
}

impl LoggerWindow {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn ui(&mut self, ui: &mut Ui) {
        let Ok(mut logs) = RECORDS_LOGGER.lock() else {
            return;
        };

        let len_log = logs.len();
        if len_log > self.max_log_len {
            logs.drain(..len_log - self.max_log_len);
        }

        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                logs.clear();
            }
            ui.menu_button("Log Levels", |ui| {
                ui.selectable_label_from_slice(&mut self.loglevels, |idx| LEVELS[idx].as_str());
            });
        });

        ui.horizontal(|ui| {
            ui.label("Search: ");
            let _response = ui.text_edit_singleline(&mut self.term);
            if ui
                .selectable_label(self.case_sensitive, "Aa")
                .on_hover_text("Case sensitive")
                .clicked()
            {
                self.case_sensitive = !self.case_sensitive;
            };
        });

        ui.horizontal(|ui| {
            ui.label("Max Log output");
            ui.add(widgets::DragValue::new(&mut self.max_log_len).speed(1));
        });

        ui.horizontal(|ui| {
            if ui.button("Sort").clicked() {
                logs.sort()
            }
        });
        ui.separator();

        let mut logs_displayed: usize = 0;

        ScrollArea::vertical()
            .auto_shrink([false, true])
            .max_height(ui.available_height() - 30.0)
            .stick_to_bottom(true)
            .show(ui, |ui_inner| {
                logs.iter()
                    .filter(|(l, s)| self.match_term(s) && (self.loglevels[*l as usize - 1]))
                    .for_each(|(lvl, s)| {
                        ui_inner.colored_label(level_color(*lvl), s);
                        logs_displayed += 1;
                    });
            });

        ui.horizontal(|ui| {
            ui.label(format!("Log size: {}", logs.len()));
            ui.label(format!("Displayed: {}", logs_displayed));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                #[allow(deprecated)]
                if ui.save_button().clicked() {
                    let homedir = env::home_dir().unwrap_or(PathBuf::from("/temp"));
                    if let Some(file) = DynoFileManager::save_file(
                        "Saving Log File",
                        "gui_log",
                        homedir,
                        &[("logfile", &["log", "log.log", "dlog"])],
                    ) {
                        if let Err(err) = fs::write(&file, logs.iter().map(|(_, s)| s).join("\n")) {
                            crate::log::error!(
                                "Failed to write to file '{}' - {err}",
                                file.display()
                            );
                        }
                    }
                }
                if ui.button("Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = logs.iter().map(|(_, s)| s).join("\n"));
                }
            });
        });

        // has to be cleared after every frame
    }

    #[inline]
    fn match_term(&self, s_term: &str) -> bool {
        if self.term.is_empty() {
            return true;
        }
        if !self.case_sensitive {
            s_term.to_lowercase().contains(&self.term.to_lowercase())
        } else {
            s_term.contains(&self.term)
        }
    }
}

/// Draws the logger ui
impl super::WindowState for LoggerWindow {
    fn show_window(
        &mut self,
        ctx: &Context,
        _control: &mut crate::control::DynoControl,
        _state: &mut crate::state::DynoState,
    ) {
        let mut open = self.open;
        Window::new("Dyno Log Window")
            .open(&mut open)
            .resizable(true)
            .id("dyno_log_window".into())
            .show(ctx, |ui| self.ui(ui));
        self.open = open;
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
