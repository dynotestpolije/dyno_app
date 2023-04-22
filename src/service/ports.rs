#![allow(unused)]

use dyno_types::{DynoErr, DynoResult};
use serialport::{SerialPortInfo, SerialPortType::UsbPort, UsbPortInfo};

#[derive(Debug, Clone, Default)]
pub struct PortInfo {
    pub port_name: String,
    pub vid: u16,
    pub pid: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}
impl PortInfo {
    #[inline(always)]
    fn from_serialport(serialport_info: SerialPortInfo) -> Option<Self> {
        let SerialPortInfo {
            port_name,
            port_type,
        } = serialport_info;
        match port_type {
            UsbPort(UsbPortInfo {
                vid,
                pid,
                serial_number,
                manufacturer,
                product,
            }) => Some(Self {
                port_name,
                vid,
                pid,
                serial_number,
                manufacturer,
                product,
            }),
            _ => None,
        }
    }

    const fn is_dyno_port(&self) -> bool {
        matches!((self.vid, self.pid), (3220, 1406))
    }
}

pub fn get_dyno_port<'err>() -> DynoResult<'err, Option<PortInfo>> {
    serialport::available_ports()
        .map(|x| {
            x.into_iter()
                .filter_map(PortInfo::from_serialport)
                .find(PortInfo::is_dyno_port)
        })
        .map_err(|err| DynoErr::input_output_error(format!("Listing Port Error: {err}")))
}
