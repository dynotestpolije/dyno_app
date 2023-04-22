mod constant;

pub mod app;
pub mod config;
pub mod control;
pub mod paths;
pub mod service;
pub mod state;
pub mod widgets;
pub mod windows;

pub use constant::*;
pub use dyno_types as types;

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

pub fn init_logger<'err>(file: impl AsRef<std::path::Path>) -> types::DynoResult<'err, ()> {
    let builder = types::LoggerBuilder::new()
        .set_file(file.as_ref().to_path_buf())
        .set_max_size(10);

    if cfg!(debug_assertions) {
        builder
            .set_max_level(types::log::LevelFilter::Debug)
            .build_console_logger()
    } else {
        builder
            .set_max_level(types::log::LevelFilter::Warn)
            .set_roll_action(types::RollAction::Roll)
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
