use std::{io::ErrorKind, sync::Arc};

use bytes::{Buf, BytesMut};
use dyno_types::{DynoErr, DynoResult, SerialData};
use eframe::epaint::mutex::Mutex;
use serialport::SerialPort;

pub const MAX_BUFFER_SIZE: usize = 1024;

pub struct Codec {
    serial: Arc<Mutex<Box<dyn SerialPort>>>,
    buffer: BytesMut,
    next_index: usize,
    is_discarding: bool,
}

impl Codec {
    pub const EOL_DELIM: u8 = b'\n';
    pub fn new(serial: Arc<Mutex<Box<dyn SerialPort>>>) -> Self {
        Self {
            serial,
            buffer: BytesMut::with_capacity(MAX_BUFFER_SIZE),
            next_index: 0,
            is_discarding: true,
        }
    }

    pub fn next_read(&mut self) -> DynoResult<Option<SerialData>> {
        let mut buffer = [0u8; MAX_BUFFER_SIZE];

        let size_read = {
            let mut lock = self.serial.lock();
            lock.read(&mut buffer[..])
        };

        match size_read {
            Ok(count) => {
                self.buffer.extend_from_slice(&buffer[..count]);
                self.decode()
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(From::from(e)),
        }
    }

    fn decode(&mut self) -> DynoResult<Option<SerialData>> {
        loop {
            // Determine how far into the buffer we'll search for a newline. If
            // there's no max_length set, we'll read to the end of the buffer.
            let size_to_read = MAX_BUFFER_SIZE.saturating_add(1).min(self.buffer.len());
            let newline_offset = self
                .buffer
                .get(self.next_index..size_to_read)
                .and_then(|b| b.iter().position(|bb| *bb == Self::EOL_DELIM));

            match (self.is_discarding, newline_offset) {
                (true, Some(offset)) => {
                    // If we found a newline, discard up to that offset and
                    // then stop discarding. On the next iteration, we'll try
                    // to read a line normally.
                    self.buffer.advance(offset + self.next_index + 1);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    // Otherwise, we didn't find a newline, so we'll discard
                    // everything we read. On the next iteration, we'll continue
                    // discarding up to max_len bytes unless we find a newline.
                    self.buffer.advance(size_to_read);
                    self.next_index = 0;
                    if self.buffer.is_empty() {
                        return Ok(None);
                    }
                }
                (false, Some(offset)) => {
                    // Found a line!
                    let newline_index = offset + self.next_index;
                    self.next_index = 0;
                    let line = self.buffer.split_to(newline_index + 1);

                    return Ok(SerialData::from_bytes(line.chunk()));
                }
                (false, None) if self.buffer.len() > MAX_BUFFER_SIZE => {
                    // Reached the maximum length without finding a
                    // newline, return an error and start discarding on the
                    // next call.
                    self.is_discarding = true;
                    return Err(DynoErr::input_output_error(
                        "input length is greather than max_lenght",
                    ));
                }
                (false, None) => {
                    // We didn't find a line or reach the length limit, so the next
                    // call will resume searching at the current offset.
                    self.next_index = size_to_read;
                    return Ok(None);
                }
            }
        }
    }
    // pub fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), String> {
    //     Ok(())
    // }
}
