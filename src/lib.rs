mod constant;

pub mod config;
pub mod paths;
pub mod service;
pub mod widgets;
pub mod window;
pub use constant::*;
pub use dyno_types as types;

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
