// #![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use dynotest_app::control::DynoControl;
use dynotest_app::{app, widgets::msgdialog::MsgDialogUnwrap, PACKAGE_INFO};
use eframe::AppCreator;

fn main() {
    let control = DynoControl::new();
    let log_dir = control.paths.get_cache_dir_file("logs/log.log");
    dynotest_app::init_logger(log_dir).msg_dialog_unwrap_default("Failed Initialize Logger!");

    dyno_core::log::info!("Running Main Windows App");
    let opt = control.app_config.app_options.main_window_opt();
    let app_creator: AppCreator = Box::new(|cc| app::Applications::new(cc, control));
    if let Err(err) = eframe::run_native(PACKAGE_INFO.app_name, opt, app_creator) {
        dyno_core::log::error!("Failed to run app eframe in native - {err}");
        if !dynotest_app::msg_dialog_err!(
            OkReport => ["Ignore the Error and close the Application", "Report the error to Developer"],
            "ERROR Running Applications",
            "Failed to running the aplication because: {err}"
        ) {
            dyno_core::log::warn!("ERROR: TODO! report the error from run native");
        }
    }
}
