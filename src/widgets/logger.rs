use std::{env, path::PathBuf};

use dyno_types::{log::Level, RECORDS_LOGGER};
use eframe::{egui, epaint::Color32};
use itertools::Itertools;

use super::{button::ButtonExt, DynoWidgets};

lazy_static::lazy_static! {
    static ref LOGGER_UI: std::sync::Mutex<LoggerUi> = Default::default();
}

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

struct LoggerUi {
    loglevels: [bool; SIZE_LEVEL],
    search_term: String,
    search_case_sensitive: bool,
    max_log_length: usize,
}

impl Default for LoggerUi {
    fn default() -> Self {
        Self {
            loglevels: [false; SIZE_LEVEL],
            search_term: String::with_capacity(128),
            search_case_sensitive: false,
            max_log_length: 1000,
        }
    }
}

impl LoggerUi {
    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let Ok(mut logs) = RECORDS_LOGGER.lock() else {
            return;
        };

        let len_log = logs.len();
        if len_log > self.max_log_length {
            logs.drain(..len_log - self.max_log_length);
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
            let _response = ui.text_edit_singleline(&mut self.search_term);
            if ui
                .selectable_label(self.search_case_sensitive, "Aa")
                .on_hover_text("Case sensitive")
                .clicked()
            {
                self.search_case_sensitive = !self.search_case_sensitive;
            };
        });

        ui.horizontal(|ui| {
            ui.label("Max Log output");
            ui.add(egui::widgets::DragValue::new(&mut self.max_log_length).speed(1));
        });

        ui.horizontal(|ui| {
            if ui.button("Sort").clicked() {
                logs.sort()
            }
        });
        ui.separator();

        let mut logs_displayed: usize = 0;

        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .max_height(ui.available_height() - 30.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                logs.iter()
                    .filter(|(l, s)| {
                        self.loglevels[*l as usize - 1]
                            && self.search_term.is_empty()
                            && self.match_string(s)
                    })
                    .for_each(|(lvl, s)| {
                        ui.colored_label(level_color(*lvl), s);
                        logs_displayed += 1;
                    });
            });

        ui.horizontal(|ui| {
            ui.label(format!("Log size: {}", logs.len()));
            ui.label(format!("Displayed: {}", logs_displayed));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                #[allow(deprecated)]
                if ui.save_button().clicked() {
                    let homedir = env::home_dir().unwrap_or(PathBuf::from("/temp"));
                    if let Some(file) = super::DynoFileManager::save_file(
                        "Saving Log File",
                        "gui_log",
                        homedir,
                        &[("logfile", &["log", "log.log", "dlog"])],
                    ) {
                        std::fs::write(file, logs.iter().map(|(_, s)| s).join("\n")).ok();
                    }
                }
            });
        });

        // has to be cleared after every frame
    }

    #[inline]
    fn match_string(&self, string: &str) -> bool {
        if !self.search_case_sensitive {
            return string
                .to_lowercase()
                .contains(&self.search_term.to_lowercase());
        }
        string.contains(&self.search_term)
    }
}

/// Draws the logger ui
/// has to be called after [`init()`](init());
#[inline(always)]
pub fn logger_ui(ui: &mut egui::Ui) {
    if let Ok(mut uilog) = LOGGER_UI.lock() {
        uilog.ui(ui)
    }
}
