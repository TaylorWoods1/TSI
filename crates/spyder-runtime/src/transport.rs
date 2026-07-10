//! Byte/line transports for motor backends.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::{Result, RuntimeError};

/// Minimal line-oriented transport used by stepper / ODrive backends.
pub trait Transport: Send {
    /// Write raw bytes (must include newline if the protocol needs it).
    fn write_all(&mut self, bytes: &[u8]) -> Result<()>;
    /// Read until newline (or timeout). Returns line without trailing `\r`/`\n`.
    fn read_line(&mut self) -> Result<String>;
}

/// Records writes; scripted replies for tests.
#[derive(Default, Debug)]
pub struct MockTransport {
    /// Bytes written (as UTF-8 lossy strings per write).
    pub writes: Vec<String>,
    /// FIFO of replies returned by [`read_line`].
    pub replies: Vec<String>,
}

impl MockTransport {
    /// Create empty mock.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a reply for the next read.
    pub fn push_reply<S: Into<String>>(&mut self, s: S) {
        self.replies.push(s.into());
    }
}

impl Transport for MockTransport {
    fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.writes
            .push(String::from_utf8_lossy(bytes).to_string());
        Ok(())
    }

    fn read_line(&mut self) -> Result<String> {
        if self.replies.is_empty() {
            return Ok("OK".into());
        }
        Ok(self.replies.remove(0))
    }
}

/// Serial port transport (`/dev/ttyUSB0`, `COM3`, …).
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
    buf: Vec<u8>,
}

impl SerialTransport {
    /// Open a serial device at `baud`.
    pub fn open(path: &str, baud: u32) -> Result<Self> {
        let port = serialport::new(path, baud)
            .timeout(Duration::from_millis(500))
            .open()
            .map_err(|e| RuntimeError::Backend(format!("serial open {path}: {e}")))?;
        Ok(Self {
            port,
            buf: Vec::new(),
        })
    }
}

impl Transport for SerialTransport {
    fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.port
            .write_all(bytes)
            .map_err(|e| RuntimeError::Backend(format!("serial write: {e}")))?;
        self.port
            .flush()
            .map_err(|e| RuntimeError::Backend(format!("serial flush: {e}")))
    }

    fn read_line(&mut self) -> Result<String> {
        let mut tmp = [0u8; 256];
        loop {
            if let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
                let line = self.buf.drain(..=pos).collect::<Vec<_>>();
                let s = String::from_utf8_lossy(&line).to_string();
                return Ok(s.trim_end_matches(['\r', '\n']).to_string());
            }
            match self.port.read(&mut tmp) {
                Ok(0) => {
                    return Err(RuntimeError::Backend("serial EOF".into()));
                }
                Ok(n) => self.buf.extend_from_slice(&tmp[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    if !self.buf.is_empty() {
                        let line = std::mem::take(&mut self.buf);
                        let s = String::from_utf8_lossy(&line).to_string();
                        return Ok(s.trim_end_matches(['\r', '\n']).to_string());
                    }
                    return Err(RuntimeError::Backend("serial read timeout".into()));
                }
                Err(e) => return Err(RuntimeError::Backend(format!("serial read: {e}"))),
            }
        }
    }
}

/// TCP transport (useful for firmware simulators / ESP32 bridges).
pub struct TcpTransport {
    stream: TcpStream,
    buf: Vec<u8>,
}

impl TcpTransport {
    /// Connect to `host:port`.
    pub fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .map_err(|e| RuntimeError::Backend(format!("tcp connect {addr}: {e}")))?;
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .ok();
        stream
            .set_write_timeout(Some(Duration::from_millis(500)))
            .ok();
        Ok(Self {
            stream,
            buf: Vec::new(),
        })
    }
}

impl Transport for TcpTransport {
    fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream
            .write_all(bytes)
            .map_err(|e| RuntimeError::Backend(format!("tcp write: {e}")))?;
        self.stream
            .flush()
            .map_err(|e| RuntimeError::Backend(format!("tcp flush: {e}")))
    }

    fn read_line(&mut self) -> Result<String> {
        let mut tmp = [0u8; 256];
        loop {
            if let Some(pos) = self.buf.iter().position(|&b| b == b'\n') {
                let line = self.buf.drain(..=pos).collect::<Vec<_>>();
                let s = String::from_utf8_lossy(&line).to_string();
                return Ok(s.trim_end_matches(['\r', '\n']).to_string());
            }
            match self.stream.read(&mut tmp) {
                Ok(0) => return Err(RuntimeError::Backend("tcp EOF".into())),
                Ok(n) => self.buf.extend_from_slice(&tmp[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Err(RuntimeError::Backend("tcp read timeout".into()));
                }
                Err(e) => return Err(RuntimeError::Backend(format!("tcp read: {e}"))),
            }
        }
    }
}
