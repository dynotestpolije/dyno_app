mod ports;
mod serial;
// pub mod api;

pub use ports::{get_dyno_port, PortInfo};
pub use serial::SerialService;

pub fn init_serial() -> Option<SerialService> {
    match SerialService::new() {
        Ok(ser) => Some(ser),
        Err(err) => {
            if !crate::msg_dialog_err!(OkCancel => ["Ignore the Error", "Exit the Aplication"],
                "Serial Port Error!",
                "{err} - Maybe because USB cable not connected properly or try restart the PROGRAM or PC"
            ) {
                crate::log::info!("manually exit by user");
                panic!()
            }
            None
        }
    }
}
