mod codec;
mod ports;
mod serial;

pub use ports::{get_dyno_port, PortInfo};
pub use serial::SerialService;

#[derive(Clone, PartialEq, Eq)]
pub enum CmdMsg {
    Noop = 0x0,
    Start = 0xB,
    Stop = 0xC,
    Restart = 0xD,
    ShutDown = 0xE,
}

impl std::fmt::Display for CmdMsg {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{own}",
            own = unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
        )
    }
}

impl CmdMsg {
    #[inline(always)]
    pub const fn as_bytes(&self) -> &'static [u8] {
        match self {
            CmdMsg::Noop => b"0:cmd\n",
            CmdMsg::Start => b"11:cmd\n",
            CmdMsg::Stop => b"12:cmd\n",
            CmdMsg::Restart => b"13:cmd\n",
            CmdMsg::ShutDown => b"14:cmd\n",
        }
    }
}

pub fn init_serial() -> Option<SerialService> {
    match SerialService::new() {
        Ok(ser) => Some(ser),
        Err(err) => {
            if !crate::msg_dialog_err!(OkCancel => ["Ignore the Error", "Exit the Aplication"],
                "Serial Port Error!",
                "{err} - Maybe because USB cable not connected properly or try restart the PROGRAM or PC"
            ) {
                dyno_types::log::info!("manually exit by user");
                panic!()
            }
            None
        }
    }
}
