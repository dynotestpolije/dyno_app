mod constant;

pub mod config;
pub mod control;
pub mod paths;
pub mod service;
pub mod state;
pub mod widgets;
pub mod windows;

pub use constant::*;
use dyno_core::{lazy_static, log, DynoErr, DynoResult, FileAction, LoggerBuilder};

pub const COLOR_BLUE_DYNO: eframe::epaint::Color32 = eframe::epaint::Color32::from_rgb(0, 204, 255);

lazy_static::lazy_static! {
    pub static ref TOAST_MSG: eframe::epaint::mutex::Mutex<widgets::toast::Toasts> = eframe::epaint::mutex::Mutex::new(widgets::toast::Toasts::new());
}

#[allow(dead_code)]
pub enum PanelId {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

impl From<PanelId> for eframe::egui::Id {
    fn from(value: PanelId) -> Self {
        match value {
            PanelId::Top => eframe::egui::Id::new("E3221406_dynotest_top_panel"),
            PanelId::Bottom => eframe::egui::Id::new("E3221406_dynotest_bottom_panel"),
            PanelId::Left => eframe::egui::Id::new("E3221406_dynotest_left_panel"),
            PanelId::Right => eframe::egui::Id::new("E3221406_dynotest_right_panel"),
            PanelId::Center => eframe::egui::Id::new("E3221406_central_panel"),
        }
    }
}

pub fn init_logger(file: impl AsRef<std::path::Path>) -> DynoResult<()> {
    let builder = LoggerBuilder::new()
        .set_file(file.as_ref().to_path_buf())
        .set_max_size(10);

    if cfg!(debug_assertions) {
        builder
            .set_max_level(log::LevelFilter::Debug)
            .build_console_logger()
    } else {
        builder
            .set_max_level(log::LevelFilter::Warn)
            .set_roll_action(FileAction::Roll)
            .build_file_logger()
    }
}

#[macro_export]
macro_rules! eq_structs {
    ($s:ident, $other:ident -> [$($varname:ident),* $(,)?]) => {
        {
            let Self { $($varname),* } = &$other;
            [ $($s.$varname.eq($varname)),* ].iter().all(|x| *x)
        }
    };
}
