//! TCP round-trip against an in-process stepper protocol server.

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use spyder_runtime::{MotorBackend, RuntimeError, StepperBackend, Transport};
use spyder_stepper_sim::SimState;

/// Wraps a transport and discards the spyder-stepper-sim welcome banner.
struct StripBanner {
    inner: Box<dyn Transport>,
    banner_drained: bool,
}

impl StripBanner {
    fn new(inner: Box<dyn Transport>) -> Self {
        Self {
            inner,
            banner_drained: false,
        }
    }

    fn drain_banner(&mut self) -> spyder_runtime::Result<()> {
        if self.banner_drained {
            return Ok(());
        }
        let line = self.inner.read_line()?;
        if line.contains("spyder-stepper-sim") {
            self.banner_drained = true;
            Ok(())
        } else {
            Err(RuntimeError::Backend(format!(
                "expected simulator banner, got {line}"
            )))
        }
    }
}

impl Transport for StripBanner {
    fn write_all(&mut self, bytes: &[u8]) -> spyder_runtime::Result<()> {
        self.inner.write_all(bytes)
    }

    fn read_line(&mut self) -> spyder_runtime::Result<String> {
        self.inner.read_line()
    }
}

fn spawn_sim_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for conn in listener.incoming() {
            if let Ok(stream) = conn {
                thread::spawn(move || {
                    let mut sim = SimState::new(4);
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut writer = stream;
                    sim.serve_connection(&mut reader, &mut writer);
                });
            }
        }
    });
    thread::sleep(Duration::from_millis(50));
    port
}

#[test]
fn stepper_backend_tcp_round_trip() {
    let port = spawn_sim_server();
    let addr = format!("127.0.0.1:{port}");
    let transport = spyder_runtime::TcpTransport::connect(&addr).unwrap();
    let mut wrapped = StripBanner::new(Box::new(transport));
    wrapped.drain_banner().unwrap();
    let mut backend = StepperBackend::new(Box::new(wrapped), 4);

    backend
        .move_steps(&[100, -25, 0, 10], &[0.001, 0.001, 0.0, 0.001])
        .unwrap();
    assert_eq!(backend.positions(), &[100, -25, 0, 10]);

    let feedback = backend.read_feedback_steps().unwrap();
    assert_eq!(feedback, vec![100, -25, 0, 10]);

    backend.home_hardware().unwrap();
    assert_eq!(backend.positions(), &[0, 0, 0, 0]);
}
