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

use crate::{toast_error, AsyncMsg};

use self::impl_serial::open_async;

#[derive(Clone)]
pub struct SerialService {
    pub info: PortInfo,
    running_flag: Arc<AtomicBool>,
}

impl SerialService {
    pub const MAX_BUFFER_SIZE: usize = 1024;
    const BAUD_RATE: u32 = 512_000;

    pub fn new() -> Option<Self> {
        let info = match ports::get_dyno_port() {
            Ok(ok) => match ok {
                Some(some) => some,
                None => {
                    toast_error!(
                        "Failed to get port info, there is no port available in this machine"
                    );
                    return None;
                }
            },
            Err(err) => {
                toast_error!("Failed to get port info, {err}");
                return None;
            }
        };
        Some(Self {
            info,
            running_flag: Arc::default(),
        })
    }

    pub fn start(&self, tx: Sender<AsyncMsg>) -> DynoResult<JoinHandle<()>> {
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
                // membaca data sampai dengan menemui Delimiter '\n',
                // dan menyimpannya pada `buffer`
                match serial_port.read_until(SerialData::DELIM, &mut buffer).await {
                    // jika tidak ada yang terbaca ( byte yang terbaca 0 ), ulang kembali loop
                    Ok(0) => continue,
                    // jika terbaca dan diakhiri delimiter, proses data tersebut
                    Ok(len) if buffer[last + len - 1] == SerialData::DELIM => {
                        // memproses data dan menkonversi byte data tersebut ke tipe data 'SerialData'
                        // dan mengirimnya melalui mpsc channel
                        if let Some(data) = SerialData::from_bytes(&buffer[..len - 1]) {
                            ignore_err!(tx.send(AsyncMsg::OnSerialData(data)))
                        }
                        // menghapus buffer, untuk menyiapkan data pada iterasi selanjutnya
                        // yang akan diterima
                        buffer.clear();
                        last = 0;
                    }
                    // jika tidak diakhiri delimiter, tambah jumlah 'last' variable dengan jumlah
                    // byte yang terbaca
                    Ok(len) => last += len,
                    // jika error
                    Err(err) => {
                        if matches!(
                            err.kind(),
                            IOEK::UnexpectedEof
                                | IOEK::TimedOut
                                | IOEK::BrokenPipe
                                | IOEK::Interrupted
                        ) {
                            continue 'loops;
                        }
                        dyno_core::log::error!("{err}");
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
