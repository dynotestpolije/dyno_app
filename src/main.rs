#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod condition;
mod controller;
mod startup;

use dynotest_app::{msg_dialog_err, types::log, widgets::msgdialog::MsgDialogUnwrap, PACKAGE_INFO};
use eframe::AppCreator;

use crate::{app::Applications, controller::Controller};

fn main() -> () {
    let controller = Controller::new();
    let log_dir = controller.paths().get_cache_dir_file("logs/log.log");
    let _log_handle = dynotest_app::init_logger(log_dir).msg_dialog_map("Initialize Error!");
    log::info!("Done Initialize Applciation");

    if let Some(opt) = controller.option_show_startup() {
        show_startup_window(opt);
    }

    log::info!("Running Main Windows App");
    let main_opt = controller.app_options();
    let main_app_creator: AppCreator = Box::new(|cc| Applications::new(cc, controller));

    if let Err(err) = eframe::run_native(PACKAGE_INFO.app_name, main_opt, main_app_creator) {
        let return_dialog = dynotest_app::msg_dialog_err!(
            OkReport => ["Ignore the Error and close the Application", "Report the error to Developer"],
            "ERROR Running Applications",
            "Failed to running the aplication because: {err}"
        );

        if !return_dialog {
            todo!("report error from dialog")
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
        let return_dialog = dynotest_app::msg_dialog_err!(
            OkReport => ["Ignore the Error", "Report the error to Developer"],
            "ERROR Running Startup Window Applications",
            "Failed to running the aplication because: {err}"
        );
        if !return_dialog {
            todo!("report error from dialog")
        }
    }
}
