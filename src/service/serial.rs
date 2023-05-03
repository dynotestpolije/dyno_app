#![allow(dead_code)]

use super::{codec::Codec, CmdMsg, PortInfo};
use dyno_types::{DynoErr, DynoResult, ResultHandler, SerialData};
use serialport::SerialPort;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver},
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
    serial: Arc<Mutex<Box<dyn SerialPort>>>,
    info: PortInfo,
    rx: Arc<Receiver<SerialData>>,
    running_flag: Arc<AtomicBool>,
}

impl SerialService {
    const BAUD_RATE: u32 = 500_000;
    pub fn new<'err>() -> DynoResult<'err, Self> {
        let (_, rx) = channel();
        let info = super::get_dyno_port()?.ok_or(DynoErr::service_error(
            "Failed to get port info, there is no port available in this machine",
        ))?;
        let serial = Arc::new(Mutex::new(
            serialport::new(&info.port_name, Self::BAUD_RATE)
                .timeout(Duration::from_millis(300))
                .open()
                .map_err(|err| DynoErr::service_error(err.to_string()))?,
        ));
        Ok(Self {
            serial,
            info,
            rx: Arc::new(rx),
            running_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(&mut self) -> DynoResult<JoinHandle<()>> {
        let (tx, rx) = channel();
        self.rx = Arc::new(rx);
        self.running_flag.store(true, Ordering::SeqCst);

        self.send(CmdMsg::Start).ok();

        let running = self.running_flag.clone();
        let serial_port = self.serial.clone();

        let serial_thread_spawn = move || {
            let mut codec = Codec::new(serial_port);
            while running.load(Ordering::SeqCst) {
                match codec.next_read() {
                    Ok(None) => continue,
                    Ok(Some(data)) => tx.send(data).ok(),
                    Err(_) => {
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                };
            }
        };
        std::thread::Builder::new()
            .name("serial_thread".to_owned())
            .spawn(serial_thread_spawn)
            .dyn_err()
    }

    pub fn stop(&mut self) {
        self.running_flag.store(false, Ordering::SeqCst);
        self.send(CmdMsg::Stop).ok();
    }

    pub fn is_open(&self) -> bool {
        self.running_flag.load(Ordering::SeqCst)
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
    pub fn handle(&self) -> DynoResult<SerialData> {
        self.rx.try_recv().map_err(|_| DynoErr::noop())
    }

    #[inline(always)]
    pub fn send(&self, cmd: CmdMsg) -> DynoResult<()> {
        match self.serial.lock() {
            Ok(mut ser) => ser.write_all(cmd.as_bytes()).dyn_err(),
            Err(_) => Err(DynoErr::noop()),
        }
    }
}
