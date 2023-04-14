// #![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod condition;
mod control;
mod startup;

use dynotest_app::{
    config::CoreConfig, msg_dialog_err, paths::DynoPaths, types::log,
    widgets::msgdialog::MsgDialogUnwrap, PACKAGE_INFO,
};
use eframe::AppCreator;

fn main() {
    let paths = match DynoPaths::new(crate::PACKAGE_INFO.app_name) {
        Ok(p) => p,
        Err(err) => {
            if !crate::msg_dialog_err!(
                OkIgnore => ["Quit the Application", "Ignore the error and continue the Application"],
                "Error Initializing Path Config",
                "cause: {err}"
            ) {
                dyno_types::log::error!("Quiting Application From error; {err}");
                std::process::exit(0);
            }
            DynoPaths::default()
        }
    };
    let config = paths
        .get_config::<CoreConfig>("config.toml")
        .unwrap_or_default();

    let log_dir = paths.get_cache_dir_file("logs/log.log");
    dynotest_app::init_logger(log_dir).msg_dialog_unwrap_default("Failed Initialize Logger!");

    if config.show_startup {
        let opt = config.app_options.startup_opt();
        show_startup_window(opt);
    }

    log::info!("Running Main Windows App");
    let opt = config.app_options.main_window_opt();
    let app_creator: AppCreator = Box::new(|cc| app::Applications::new(cc, paths, config));

    if let Err(err) = eframe::run_native(PACKAGE_INFO.app_name, opt, app_creator) {
        log::error!("Failed to run app eframe in native - {err}");
        if !dynotest_app::msg_dialog_err!(
            OkReport => ["Ignore the Error and close the Application", "Report the error to Developer"],
            "ERROR Running Applications",
            "Failed to running the aplication because: {err}"
        ) {
            todo!("Reporting Error")
        }
    }
}

fn show_startup_window(native_opt: eframe::NativeOptions) {
    let name = format!("welcome to {name}", name = PACKAGE_INFO.app_name);
    if let Err(err) = eframe::run_native(
        name.as_str(),
        native_opt,
        Box::new(|cc| startup::StartupWindow::new(cc)),
    ) {
        if !dynotest_app::msg_dialog_err!(
            OkReport => ["Ignore the Error", "Report the error to Developer"],
            "ERROR Running Startup Window Applications",
            "Failed to running the aplication because: {err}"
        ) {
            todo!("Reporting Error")
        }
    }
}
