#![allow(unused)]

use crate::{DynoErr, DynoResult};
use serialport::{
    SerialPortInfo,
    SerialPortType::{PciPort, UsbPort},
    UsbPortInfo,
};

#[derive(Debug, Clone, Default)]
pub struct PortInfo {
    pub port_name: String,
    pub vid: u16,
    pub pid: u16,
}
impl PortInfo {
    /// construct from [SerialPortInfo] into [Self]
    #[inline]
    fn from_serialport(
        SerialPortInfo {
            port_name,
            port_type,
        }: SerialPortInfo,
    ) -> Option<Self> {
        match port_type {
            UsbPort(UsbPortInfo { vid, pid, .. }) => Some(Self {
                port_name,
                vid,
                pid,
            }),
            PciPort if port_name.contains("/dev/tty") => Some(Self {
                port_name,
                vid: 3220,
                pid: 1406,
            }),
            _ => None,
        }
    }

    /// check if [Self] is dynotests device port
    const fn is_dyno_port(&self) -> bool {
        matches!((self.vid, self.pid), (3220, 1406))
    }
}

pub fn get_dyno_port() -> DynoResult<Option<PortInfo>> {
    serialport::available_ports()
        .map(|x| {
            x.into_iter()
                .filter_map(PortInfo::from_serialport)
                .find(PortInfo::is_dyno_port)
        })
        .map_err(|err| DynoErr::input_output_error(format!("Failed Getting Port: {err}")))
}
