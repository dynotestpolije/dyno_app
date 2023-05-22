#![allow(dead_code)]
mod impl_serial;
pub mod ports;

use crossbeam_channel::{unbounded, Receiver, Sender};
use dyno_core::{
    ignore_err, log,
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

use self::impl_serial::open_async;

#[derive(Clone, PartialEq, Eq)]
enum SerialFlag {
    Idle,
    Connected,
    Disconnected,
    Error,
}

#[derive(Clone)]
pub struct SerialService {
    pub info: PortInfo,

    rx: Receiver<Option<SerialData>>,
    tx: Sender<Option<SerialData>>,
    running_flag: Arc<AtomicBool>,
}

impl SerialService {
    pub const MAX_BUFFER_SIZE: usize = 1024;
    const BAUD_RATE: u32 = 512_000;
    pub fn new() -> DynoResult<Self> {
        let (tx, rx) = unbounded();
        let info = ports::get_dyno_port()?.ok_or(DynoErr::service_error(
            "Failed to get port info, there is no port available in this machine",
        ))?;
        Ok(Self {
            info,
            tx,
            rx,
            running_flag: Arc::default(),
        })
    }

    pub fn start(&mut self) -> DynoResult<JoinHandle<()>> {
        if self.running_flag.load(Ordering::Relaxed) {
            return Err(DynoErr::service_error("Serial Service Already Running"));
        }
        self.running_flag.store(true, Ordering::Relaxed);

        let running = self.running_flag.clone();
        let mut serial_port = BufReader::new(open_async(&self.info.port_name, Self::BAUD_RATE)?);
        let tx = self.tx.clone();

        let serial_thread_spawn = async move {
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
                        ignore_err!(tx.send(SerialData::from_bytes(&buffer[..len - 1])));
                        buffer.clear();
                        last = 0;
                    }
                    Ok(len) => last += len,
                    Err(err) => {
                        if matches!(err.kind(), IOEK::UnexpectedEof | IOEK::TimedOut) {
                            continue 'loops;
                        }
                        log::error!("ERROR: SerialPort Reads - ({err})");
                        running.store(false, Ordering::Relaxed);
                    }
                }
            }
            drop(serial_port);
        };

        Ok(dyno_core::tokio::spawn(serial_thread_spawn))
    }

    pub fn stop(&mut self) {
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

impl SerialService {
    #[inline(always)]
    pub fn handle(&self) -> Option<SerialData> {
        self.rx.try_recv().ok().flatten()
    }
}
