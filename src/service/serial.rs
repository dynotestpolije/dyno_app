#![allow(dead_code)]

use super::PortInfo;
use dyno_core::{ignore_err, log, DynoErr, DynoResult, ResultHandler, SerialData};
use serialport::SerialPort;
use std::{
    io::{BufRead, BufReader, ErrorKind as IOEK},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

type SerialPortDyno = Arc<Mutex<Box<dyn SerialPort>>>;
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

    rx: Arc<Receiver<Option<SerialData>>>,
    tx: Sender<Option<SerialData>>,
    running_flag: Arc<AtomicBool>,
}

impl SerialService {
    pub const MAX_BUFFER_SIZE: usize = 1024;
    const BAUD_RATE: u32 = 512_000;
    pub fn new() -> DynoResult<Self> {
        let (tx, rx) = channel();
        let info = super::get_dyno_port()?.ok_or(DynoErr::service_error(
            "Failed to get port info, there is no port available in this machine",
        ))?;
        Ok(Self {
            info,
            tx,
            rx: Arc::new(rx),
            running_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(&mut self) -> DynoResult<JoinHandle<()>> {
        self.running_flag.store(true, Ordering::Relaxed);

        let running = self.running_flag.clone();
        let mut serial_port = BufReader::with_capacity(Self::MAX_BUFFER_SIZE, {
            let mut ser = serialport::new(&self.info.port_name, Self::BAUD_RATE)
                .timeout(Duration::from_millis(300))
                .open()
                .map_err(|err| DynoErr::serial_port_error(err.to_string()))?;
            ignore_err!(ser.write(&b"cmd:1\n"[..]));
            ser
        });
        let tx = self.tx.clone();

        let serial_thread_spawn = move || {
            // let mut codec = Cod;c::new(serial_port);
            let mut last: usize = 0;
            let mut buffer: Vec<u8> = Vec::with_capacity(SerialData::SIZE * 2);

            'loops: loop {
                if !running.load(Ordering::Relaxed) {
                    break 'loops;
                }
                match serial_port.read_until(SerialData::DELIM, &mut buffer) {
                    Ok(len) if buffer[last + len - 1] == SerialData::DELIM => {
                        ignore_err!(tx.send(SerialData::from_bytes(&buffer[..len - 1])));
                        buffer.clear();
                        last = 0;
                    }
                    Ok(len) => {
                        last += len;
                        if last >= buffer.len() {
                            buffer.clear();
                            last = buffer.len();
                        }
                    }
                    Err(err) => {
                        if matches!(err.kind(), IOEK::WouldBlock | IOEK::Interrupted) {
                            continue 'loops;
                        }
                        log::error!("ERROR: SerialPort Reads - {err}");
                        running.store(false, Ordering::Relaxed);
                    }
                }
            }
            drop(serial_port);
            drop(buffer);
        };
        std::thread::Builder::new()
            .name("serial_thread".to_owned())
            .spawn(serial_thread_spawn)
            .dyn_err()
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
