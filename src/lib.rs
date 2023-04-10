mod constant;

pub mod config;
pub mod paths;
pub mod service;
pub mod widgets;
pub mod window;
pub use constant::*;
pub use dyno_types as types;

// const SIZE_TRIGGER_ROLLING_FILE: u64 = 50 * 1024 * 1024; // 50Mb

pub fn init_logger<'err>(file: impl AsRef<std::path::Path>) -> types::DynoResult<'err, ()> {
    types::log_to_file(file, types::log::LevelFilter::Info, true)
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
