use eframe::{
    epaint::{Pos2, Vec2},
    IconData, NativeOptions, Theme,
};

use crate::{
    assets::ICO_LOGO,
    open_option_icon, row_label_value,
    widgets::{DisplayStylePreset, DynoWidgets},
};
use dyno_core::serde;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(crate = "serde")]
pub struct ApplicationConfig {
    pub segment_display_style: DisplayStylePreset,
    pub app_options: AppOptions,
    pub show_startup: bool,
}

impl ApplicationConfig {
    pub fn check_is_changed(&mut self, other: &Self) {
        if !self.app_options.eq(&other.app_options) && self.show_startup != other.show_startup {
            *self = other.clone();
        }
    }
    pub fn draw(&mut self, ui: &mut eframe::egui::Ui) {
        ui.checkbox(&mut self.show_startup, "Show Startup Window");
        ui.separator();
        self.app_options.ui(ui);

        let iter = self.segment_display_style.get_iter();
        ui.combobox_from_iter(
            "Style for SevenSegment",
            &mut self.segment_display_style,
            iter,
        );
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, crate = "serde")]
pub struct AppOptions {
    pub icon_path: Option<String>,
    pub always_on_top: bool,
    pub maximized: bool,
    pub decorated: bool,
    pub fullscreen: bool,
    pub drag_and_drop_support: bool,
    pub initial_window_pos: Option<Pos2>,
    pub initial_window_size: Option<Vec2>,
    pub min_window_size: Option<Vec2>,
    pub max_window_size: Option<Vec2>,
    pub resizable: bool,
    pub transparent: bool,
    pub vsync: bool,
    pub multisampling: u16,
    pub depth_buffer: u8,
    pub stencil_buffer: u8,
    pub follow_system_theme: bool,
    pub default_theme: Theme,
    pub run_and_return: bool,
}

impl AppOptions {
    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        use eframe::egui::{Grid, RichText};
        ui.heading("Application Options Setting");

        let ui_grid_appoptions = |ui: &mut eframe::egui::Ui| {
            row_label_value!(
                ui => ui.optional_value_widget(&mut self.icon_path, |ui, value| {
                    let text = RichText::new(value.to_string())
                        .background_color(ui.visuals().extreme_bg_color);
                    let response = ui.link(text).on_hover_text("Left Click to Edit");
                    if response.clicked() {
                        if let Some(path) =
                            crate::widgets::DynoFileManager::pick_folder("Change Path", &value)
                        {
                            *value = path.display().to_string();
                        }
                    }
                    response
                }),
                "Icon Path",
                "icon for aplication, default to icon thath saved memory to (embed into app)"
            );
            ui.end_row();
            row_label_value!(ui => ui.toggle(&mut self.always_on_top),
                    "AlwaysOnTop",
                    "Aplication Config, Always on top window config parameters");
            row_label_value!(ui => ui.toggle(&mut self.maximized),
                    "Maximize",
                    "Aplication Config, Maximize window config parameters");
            ui.end_row();
            row_label_value!(ui => ui.toggle(&mut self.decorated),
                    "Decorated",
                    "Aplication Config, Decorated window config parameters");
            row_label_value!(ui => ui.toggle(&mut self.fullscreen),
                    "Fullscreen",
                    "Aplication Config, Fullscreen window config parameters");
            ui.end_row();
            row_label_value!(ui => ui.toggle(&mut self.drag_and_drop_support),
                    "Drag and Drop",
                    "Aplication Config, Drag and Drop Support window config parameters");
            row_label_value!(ui => ui.toggle(&mut self.resizable),
                    "Resizeable",
                    "Aplication Config, Resizeable Support window config parameters");
            ui.end_row();
            row_label_value!(ui => ui.toggle(&mut self.follow_system_theme),
                    "Follow System Theme",
                    "Aplication Config, Follow System Theme window config parameters");
        };

        Grid::new("_grid_config_edit")
            .striped(false)
            .num_columns(4)
            .show(ui, ui_grid_appoptions)
            .response
    }
}

impl PartialEq for AppOptions {
    fn eq(&self, other: &Self) -> bool {
        crate::eq_structs!(self, other -> [
            icon_path,
            always_on_top,
            maximized,
            decorated,
            fullscreen,
            drag_and_drop_support,
            initial_window_pos,
            initial_window_size,
            min_window_size,
            max_window_size,
            resizable,
            transparent,
            vsync,
            multisampling,
            depth_buffer,
            stencil_buffer,
            follow_system_theme,
            default_theme,
            run_and_return
        ])
    }
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            icon_path: None,
            always_on_top: false,
            maximized: false,
            decorated: true,
            fullscreen: false,
            min_window_size: Some(eframe::epaint::Vec2 {
                x: 720f32,
                y: 480f32,
            }),
            max_window_size: None,
            resizable: true,
            drag_and_drop_support: true,
            initial_window_pos: None,
            initial_window_size: None,
            transparent: false,
            vsync: true,
            multisampling: 0,
            depth_buffer: 0,
            stencil_buffer: 0,
            follow_system_theme: cfg!(target_os = "macos") || cfg!(target_os = "windows"),
            default_theme: Theme::Dark,
            run_and_return: true,
        }
    }
}

impl AppOptions {
    pub fn startup_opt(&self) -> NativeOptions {
        let s = self.clone().into_native_options();
        NativeOptions {
            always_on_top: true,
            resizable: false,
            initial_window_size: Some(Vec2 {
                x: 720f32,
                y: 480f32,
            }),
            ..s
        }
    }
    pub fn main_window_opt(&self) -> NativeOptions {
        self.clone().into_native_options()
    }

    fn into_native_options(self) -> NativeOptions {
        let Self {
            always_on_top,
            maximized,
            decorated,
            fullscreen,
            drag_and_drop_support,
            initial_window_pos,
            initial_window_size,
            min_window_size,
            max_window_size,
            resizable,
            transparent,
            vsync,
            multisampling,
            depth_buffer,
            stencil_buffer,
            follow_system_theme,
            default_theme,
            run_and_return,
            icon_path,
        } = self;

        let icon_data = match icon_path.map(std::path::PathBuf::from) {
            Some(path) if path.exists() => open_option_icon!(path),
            _ => ICO_LOGO.as_ref().map(|(icon, _)| icon.clone()),
        };

        eframe::NativeOptions {
            always_on_top,
            maximized,
            decorated,
            fullscreen,
            drag_and_drop_support,
            initial_window_pos,
            initial_window_size,
            min_window_size,
            max_window_size,
            resizable,
            transparent,
            vsync,
            multisampling,
            depth_buffer,
            stencil_buffer,
            follow_system_theme,
            default_theme,
            run_and_return,
            icon_data,
            ..Default::default()
        }
    }
}
