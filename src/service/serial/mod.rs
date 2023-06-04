#![allow(dead_code)]
mod impl_serial;
pub mod ports;

use dyno_core::{
    crossbeam_channel::Sender,
    ignore_err,
    tokio::{
        io::{AsyncBufReadExt, BufReader, ErrorKind as IOEK},
        task::JoinHandle,
    },
    DynoErr, DynoResult, SerialData,
};
use ports::PortInfo;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::AsyncMsg;

use self::impl_serial::open_async;

#[derive(Clone)]
pub struct SerialService {
    pub info: PortInfo,
    running_flag: Arc<AtomicBool>,
}

impl SerialService {
    pub const MAX_BUFFER_SIZE: usize = 1024;
    const BAUD_RATE: u32 = 512_000;

    pub fn new() -> DynoResult<Self> {
        let info = ports::get_dyno_port()?.ok_or(DynoErr::service_error(
            "Failed to get port info, there is no port available in this machine",
        ))?;
        Ok(Self {
            info,
            running_flag: Arc::default(),
        })
    }

    pub fn start(&mut self, tx: Sender<AsyncMsg>) -> DynoResult<JoinHandle<()>> {
        if self.running_flag.load(Ordering::Relaxed) {
            return Err(DynoErr::service_error("Serial Service Already Running"));
        }
        self.running_flag.store(true, Ordering::Relaxed);

        let running = self.running_flag.clone();
        let port_name = self.info.port_name.clone();
        let serial_inner = open_async(port_name, Self::BAUD_RATE)?;

        let serial_thread_spawn = async move {
            let mut serial_port = BufReader::new(serial_inner);
            // let mut codec = Cod;c::new(serial_port);
            let mut last: usize = 0;
            let mut buffer: Vec<u8> = Vec::with_capacity(SerialData::SIZE * 2);

            'loops: loop {
                if !running.load(Ordering::Relaxed) {
                    break 'loops;
                }
                match serial_port.read_until(SerialData::DELIM, &mut buffer).await {
                    Ok(0) => continue,
                    Ok(len) if buffer[last + len - 1] == SerialData::DELIM => {
                        if let Some(data) = SerialData::from_bytes(&buffer[..len - 1]) {
                            ignore_err!(tx.send(AsyncMsg::OnSerialData(data)))
                        }
                        buffer.clear();
                        last = 0;
                    }
                    Ok(len) => last += len,
                    Err(err) => {
                        if matches!(err.kind(), IOEK::UnexpectedEof | IOEK::TimedOut) {
                            continue 'loops;
                        }
                        ignore_err!(tx.send(AsyncMsg::error(err)));
                        running.store(false, Ordering::Relaxed);
                    }
                }
            }
            drop(serial_port);
        };

        Ok(dyno_core::tokio::spawn(serial_thread_spawn))
    }

    pub fn stop(&self) {
        self.running_flag.store(false, Ordering::Relaxed);
    }

    pub fn is_open(&self) -> bool {
        self.running_flag.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn get_info(&self) -> &PortInfo {
        &self.info
    }

    pub const fn get_baudrate(&self) -> u32 {
        Self::BAUD_RATE
    }
}
