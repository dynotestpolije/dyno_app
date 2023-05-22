use dyno_core::{tokio, DynoErr, DynoResult};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use std::{
    io::{Read, Result as IoResult, Write},
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(unix)]
mod os_prelude {
    pub use super::tokio::io::unix::AsyncFd;
    pub use futures::ready;
    pub use serialport::{BreakDuration, TTYPort};
}

#[cfg(windows)]
mod os_prelude {
    pub use super::tokio::net::windows::named_pipe::NamedPipeClient;
    pub use serialport::COMPort;
    pub use std::mem;
    pub use std::ops::{Deref, DerefMut};
    pub use std::os::windows::prelude::*;
}

#[cfg(unix)]
type PortType = TTYPort;
#[cfg(windows)]
type PortType = COMPort;

use os_prelude::*;

#[derive(Debug)]
pub struct SerialStream {
    #[cfg(unix)]
    inner: AsyncFd<PortType>,
    #[cfg(windows)]
    inner: NamedPipeClient,
    #[cfg(windows)]
    com: mem::ManuallyDrop<COMPort>,
}

impl SerialStream {
    /// Open serial port from a provided path, using the default reactor.
    pub fn open(builder: &serialport::SerialPortBuilder) -> DynoResult<Self> {
        let port = PortType::open(builder)
            .map_err(|err| DynoErr::service_error(format!("Error On Opeing Port - ({err})")))?;
        #[cfg(unix)]
        {
            Ok(Self {
                inner: AsyncFd::new(port)?,
            })
        }

        #[cfg(windows)]
        {
            let handle = port.as_raw_handle();
            // Keep the com port around to use for serialport related things
            let com = mem::ManuallyDrop::new(port);
            Ok(Self {
                inner: unsafe {
                    NamedPipeClient::from_raw_handle(handle)
                        .map_err(|err| DynoErr::service_error(format!("{err}")))?
                },
                com,
            })
        }
    }

    #[cfg(unix)]
    pub fn pair() -> DynoResult<(Self, Self)> {
        let (master, slave) = TTYPort::pair()
            .map_err(|err| DynoErr::service_error(format!("Error On Pairing TTYPort - ({err})")))?;

        let master = SerialStream {
            inner: AsyncFd::new(master).map_err(|err| DynoErr::service_error(format!("{err}")))?,
        };
        let slave = SerialStream {
            inner: AsyncFd::new(slave).map_err(|err| DynoErr::service_error(format!("{err}")))?,
        };
        Ok((master, slave))
    }

    #[cfg(unix)]
    pub fn set_exclusive(&mut self, exclusive: bool) -> DynoResult<()> {
        self.inner
            .get_mut()
            .set_exclusive(exclusive)
            .map_err(|err| {
                DynoErr::service_error(format!("Error On Set exclusive in port - ({err})"))
            })
    }

    #[cfg(unix)]
    pub fn exclusive(&self) -> bool {
        self.inner.get_ref().exclusive()
    }

    pub fn try_read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        #[cfg(unix)]
        {
            self.inner.get_mut().read(buf)
        }
        #[cfg(windows)]
        {
            self.inner.try_read(buf)
        }
    }

    pub async fn readable(&self) -> IoResult<()> {
        let _ = self.inner.readable().await?;
        Ok(())
    }

    pub fn try_write(&mut self, buf: &[u8]) -> IoResult<usize> {
        #[cfg(unix)]
        {
            self.inner.get_mut().write(buf)
        }
        #[cfg(windows)]
        {
            self.inner.try_write(buf)
        }
    }

    pub async fn writable(&self) -> IoResult<()> {
        let _ = self.inner.writable().await?;
        Ok(())
    }

    #[inline(always)]
    pub fn borrow(&self) -> &impl serialport::SerialPort {
        #[cfg(unix)]
        {
            self.inner.get_ref()
        }
        #[cfg(windows)]
        {
            self.com.deref()
        }
    }

    #[inline(always)]
    pub fn borrow_mut(&mut self) -> &mut TTYPort {
        #[cfg(unix)]
        {
            self.inner.get_mut()
        }
        #[cfg(windows)]
        {
            self.com.deref_mut()
        }
    }
}

#[cfg(unix)]
impl AsyncRead for SerialStream {
    /// Attempts to ready bytes on the serial port.
    ///
    /// # Return value
    ///
    /// The function returns:
    ///
    /// * `Poll::Pending` if the socket is not ready to read
    /// * `Poll::Ready(Ok(()))` reads data `ReadBuf` if the socket is ready
    /// * `Poll::Ready(Err(e))` if an error is encountered.
    ///
    /// # Errors
    ///
    /// This function may encounter any standard I/O error except `WouldBlock`.
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        loop {
            let mut guard = ready!(self.inner.poll_read_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().read(buf.initialize_unfilled())) {
                Ok(Ok(bytes_read)) => {
                    buf.advance(bytes_read);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => {
                    return Poll::Ready(Err(err));
                }
                Err(_would_block) => continue,
            }
        }
    }
}

#[cfg(unix)]
impl AsyncWrite for SerialStream {
    /// Attempts to send data on the serial port
    ///
    /// Note that on multiple calls to a `poll_*` method in the send direction,
    /// only the `Waker` from the `Context` passed to the most recent call will
    /// be scheduled to receive a wakeup.
    ///
    /// # Return value
    ///
    /// The function returns:
    ///
    /// * `Poll::Pending` if the socket is not available to write
    /// * `Poll::Ready(Ok(n))` `n` is the number of bytes sent
    /// * `Poll::Ready(Err(e))` if an error is encountered.
    ///
    /// # Errors
    ///
    /// This function may encounter any standard I/O error except `WouldBlock`.
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;
            match guard.try_io(|inner| inner.get_mut().flush()) {
                Ok(_) => return Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let _ = self.poll_flush(cx)?;
        Ok(()).into()
    }
}

#[cfg(windows)]
impl AsyncRead for SerialStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let mut self_ = self;
        Pin::new(&mut self_.inner).poll_read(cx, buf)
    }
}

#[cfg(windows)]
impl AsyncWrite for SerialStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        let mut self_ = self;
        Pin::new(&mut self_.inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let mut self_ = self;
        Pin::new(&mut self_.inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let mut self_ = self;
        Pin::new(&mut self_.inner).poll_shutdown(cx)
    }
}

impl serialport::SerialPort for SerialStream {
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.borrow().name()
    }

    #[inline(always)]
    fn baud_rate(&self) -> serialport::Result<u32> {
        self.borrow().baud_rate()
    }

    #[inline(always)]
    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        self.borrow().data_bits()
    }

    #[inline(always)]
    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        self.borrow().flow_control()
    }

    #[inline(always)]
    fn parity(&self) -> serialport::Result<serialport::Parity> {
        self.borrow().parity()
    }

    #[inline(always)]
    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        self.borrow().stop_bits()
    }

    #[inline(always)]
    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(0)
    }

    #[inline(always)]
    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        self.borrow_mut().set_baud_rate(baud_rate)
    }

    #[inline(always)]
    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        self.borrow_mut().set_data_bits(data_bits)
    }

    #[inline(always)]
    fn set_flow_control(
        &mut self,
        flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        self.borrow_mut().set_flow_control(flow_control)
    }

    #[inline(always)]
    fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
        self.borrow_mut().set_parity(parity)
    }

    #[inline(always)]
    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
        self.borrow_mut().set_stop_bits(stop_bits)
    }

    #[inline(always)]
    fn set_timeout(&mut self, _: std::time::Duration) -> serialport::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn write_request_to_send(&mut self, level: bool) -> serialport::Result<()> {
        self.borrow_mut().write_request_to_send(level)
    }

    #[inline(always)]
    fn write_data_terminal_ready(&mut self, level: bool) -> serialport::Result<()> {
        self.borrow_mut().write_data_terminal_ready(level)
    }

    #[inline(always)]
    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        self.borrow_mut().read_clear_to_send()
    }

    #[inline(always)]
    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        self.borrow_mut().read_data_set_ready()
    }

    #[inline(always)]
    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        self.borrow_mut().read_ring_indicator()
    }

    #[inline(always)]
    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        self.borrow_mut().read_carrier_detect()
    }

    #[inline(always)]
    fn bytes_to_read(&self) -> serialport::Result<u32> {
        self.borrow().bytes_to_read()
    }

    #[inline(always)]
    fn bytes_to_write(&self) -> serialport::Result<u32> {
        self.borrow().bytes_to_write()
    }

    #[inline(always)]
    fn clear(&self, buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        self.borrow().clear(buffer_to_clear)
    }

    #[inline(always)]
    fn try_clone(&self) -> serialport::Result<Box<dyn serialport::SerialPort>> {
        Err(serialport::Error::new(
            serialport::ErrorKind::InvalidInput,
            "Cannot clone Tokio handles",
        ))
    }

    #[inline(always)]
    fn set_break(&self) -> serialport::Result<()> {
        self.borrow().set_break()
    }

    #[inline(always)]
    fn clear_break(&self) -> serialport::Result<()> {
        self.borrow().clear_break()
    }
}

impl Read for SerialStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.try_read(buf)
    }
}

impl Write for SerialStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.try_write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.borrow_mut().flush()
    }
}

#[cfg(unix)]
mod sys {
    use super::SerialStream;
    use std::os::unix::io::{AsRawFd, RawFd};
    impl AsRawFd for SerialStream {
        fn as_raw_fd(&self) -> RawFd {
            self.inner.as_raw_fd()
        }
    }
}

#[cfg(windows)]
mod io {
    use super::SerialStream;
    use std::os::windows::io::{AsRawHandle, RawHandle};
    impl AsRawHandle for SerialStream {
        fn as_raw_handle(&self) -> RawHandle {
            self.inner.as_raw_handle()
        }
    }
}

/// An extension trait for serialport::SerialPortBuilder
/// This trait adds one method to SerialPortBuilder:
///
/// - open_native_async
/// This method mirrors the `open_native` method of SerialPortBuilder
pub trait SerialPortBuilderExt {
    /// Open a platform-specific interface to the port with the specified settings
    fn open_native_async(self) -> DynoResult<SerialStream>;
}

impl SerialPortBuilderExt for serialport::SerialPortBuilder {
    /// Open a platform-specific interface to the port with the specified settings
    fn open_native_async(self) -> DynoResult<SerialStream> {
        SerialStream::open(&self)
    }
}

pub fn open_async<'a>(
    path: impl Into<std::borrow::Cow<'a, str>>,
    baud_rate: u32,
) -> DynoResult<SerialStream> {
    serialport::new(path, baud_rate).open_native_async()
}
