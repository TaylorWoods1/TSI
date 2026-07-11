//! Dedicated hardware thread owning serial/TCP transports.
//!
//! Real `MotorBackend` instances may not be `Send`, but Axum state must be
//! `Send + Sync`. [`ChannelMotorBackend`] proxies commands over `std::sync::mpsc`
//! so transports live on a worker thread.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use spyder_runtime::{
    AxisMap, MockBackend, MotorBackend, MultiBoardBackend, ODriveAxis, ODriveBackend,
    RuntimeError, SerialTransport, StepperBackend, TcpTransport, Transport,
};

type HwResult<T> = Result<T, String>;

enum HwRequest {
    MoveSteps {
        steps: Vec<i64>,
        delays: Vec<f64>,
        reply: Sender<HwResult<()>>,
    },
    ReadFeedback {
        reply: Sender<HwResult<Vec<i64>>>,
    },
    Estop {
        reply: Sender<HwResult<()>>,
    },
    Home {
        reply: Sender<HwResult<()>>,
    },
    Shutdown,
}

struct HwWorker {
    tx: Sender<HwRequest>,
    positions: Arc<Mutex<Vec<i64>>>,
    join: Option<JoinHandle<()>>,
}

impl HwWorker {
    fn spawn<F>(name: &str, build: F) -> HwResult<(Self, usize)>
    where
        F: FnOnce() -> HwResult<Box<dyn MotorBackend>> + Send + 'static,
    {
        let (req_tx, req_rx) = mpsc::channel::<HwRequest>();
        let positions = Arc::new(Mutex::new(Vec::<i64>::new()));
        let positions_worker = Arc::clone(&positions);
        let (init_tx, init_rx) = mpsc::channel::<HwResult<usize>>();

        let join = thread::Builder::new()
            .name(format!("spyder-hw-{name}"))
            .spawn(move || {
                let result = (|| -> HwResult<()> {
                    let mut backend = build()?;
                    let n = backend.axis_count();
                    if let Ok(mut p) = positions_worker.lock() {
                        p.clear();
                        p.extend_from_slice(backend.positions());
                    }
                    init_tx.send(Ok(n)).ok();
                    hw_loop(&mut backend, req_rx, positions_worker);
                    Ok(())
                })();
                if let Err(e) = result {
                    let _ = init_tx.send(Err(e));
                }
            })
            .map_err(|e| format!("spawn hw thread: {e}"))?;

        let axis_count = init_rx
            .recv()
            .map_err(|e| format!("hw init recv: {e}"))??;

        Ok((
            Self {
                tx: req_tx,
                positions,
                join: Some(join),
            },
            axis_count,
        ))
    }

    fn request<T>(&self, build: impl FnOnce(Sender<HwResult<T>>) -> HwRequest) -> HwResult<T> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx
            .send(build(reply_tx))
            .map_err(|e| format!("hw channel: {e}"))?;
        reply_rx
            .recv()
            .map_err(|e| format!("hw reply: {e}"))?
    }

    fn mirrored_positions(&self) -> Vec<i64> {
        self.positions
            .lock()
            .map(|p| p.clone())
            .unwrap_or_default()
    }
}

impl Drop for HwWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(HwRequest::Shutdown);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

fn hw_loop(
    backend: &mut Box<dyn MotorBackend>,
    req_rx: Receiver<HwRequest>,
    positions: Arc<Mutex<Vec<i64>>>,
) {
    while let Ok(req) = req_rx.recv() {
        match req {
            HwRequest::MoveSteps {
                steps,
                delays,
                reply,
            } => {
                let result = backend.move_steps(&steps, &delays).map_err(|e| e.to_string());
                if result.is_ok() {
                    if let Ok(mut p) = positions.lock() {
                        p.clear();
                        p.extend_from_slice(backend.positions());
                    }
                }
                let _ = reply.send(result);
            }
            HwRequest::ReadFeedback { reply } => {
                let result = backend.read_feedback_steps().map_err(|e| e.to_string());
                if let Ok(ref fb) = result {
                    if let Ok(mut p) = positions.lock() {
                        *p = fb.clone();
                    }
                }
                let _ = reply.send(result);
            }
            HwRequest::Estop { reply } => {
                let _ = reply.send(backend.estop().map_err(|e| e.to_string()));
            }
            HwRequest::Home { reply } => {
                let result = backend.home_hardware().map_err(|e| e.to_string());
                if result.is_ok() {
                    if let Ok(mut p) = positions.lock() {
                        p.clear();
                        p.extend_from_slice(backend.positions());
                    }
                }
                let _ = reply.send(result);
            }
            HwRequest::Shutdown => break,
        }
    }
}

/// Send-safe motor backend proxy.
pub struct ChannelMotorBackend {
    worker: HwWorker,
    positions_cache: Vec<i64>,
    axis_count: usize,
}

impl ChannelMotorBackend {
    /// In-process mock with `n` axes.
    pub fn mock(n: usize) -> HwResult<Self> {
        Self::from_builder("mock", move || Ok(Box::new(MockBackend::new(n))))
    }

    /// Stepper over serial path or TCP `host:port`.
    pub fn stepper(device: &str, baud: u32, n: usize) -> HwResult<Self> {
        let device = device.to_string();
        Self::from_builder("stepper", move || {
            let mut transport = open_transport(&device, baud)?;
            let _ = transport.read_line();
            Ok(Box::new(StepperBackend::new(transport, n)))
        })
    }

    /// ODrive ASCII backend.
    pub fn odrive(device: &str, baud: u32, n: usize) -> HwResult<Self> {
        let device = device.to_string();
        Self::from_builder("odrive", move || {
            let transport = open_transport(&device, baud)?;
            let oaxes: Vec<_> = (0..n)
                .map(|i| ODriveAxis::new((i % 2) as u8, 200.0))
                .collect();
            let mut backend = ODriveBackend::new(transport, oaxes);
            backend.enter_closed_loop().map_err(|e| e.to_string())?;
            Ok(Box::new(backend))
        })
    }

    /// Multi-board fan-out; `mock` uses in-process mocks per device.
    pub fn multiboard(map: AxisMap, mock: bool) -> HwResult<Self> {
        if mock {
            return Self::from_builder("multiboard-mock", move || {
                Ok(Box::new(
                    MultiBoardBackend::mock_from_map(map).map_err(|e| e.to_string())?,
                ))
            });
        }
        Self::from_builder("multiboard", move || {
            let devices = map.devices();
            let mut boards: Vec<Box<dyn MotorBackend>> = Vec::new();
            for device in &devices {
                let baud = map
                    .cables
                    .iter()
                    .find(|c| &c.device == device)
                    .map(|c| c.baud)
                    .unwrap_or(115_200);
                let n_local = map.cables.iter().filter(|c| &c.device == device).count();
                let mut transport = open_transport(device, baud)?;
                let _ = transport.read_line();
                boards.push(Box::new(StepperBackend::new(transport, n_local)));
            }
            Ok(Box::new(
                MultiBoardBackend::new(map, boards).map_err(|e| e.to_string())?,
            ))
        })
    }

    fn from_builder<F>(name: &str, build: F) -> HwResult<Self>
    where
        F: FnOnce() -> HwResult<Box<dyn MotorBackend>> + Send + 'static,
    {
        let (worker, axis_count) = HwWorker::spawn(name, build)?;
        let positions_cache = worker.mirrored_positions();
        Ok(Self {
            worker,
            positions_cache,
            axis_count,
        })
    }

    fn sync_positions(&mut self) {
        self.positions_cache = self.worker.mirrored_positions();
        if self.positions_cache.len() != self.axis_count {
            self.positions_cache.resize(self.axis_count, 0);
        }
    }
}

impl MotorBackend for ChannelMotorBackend {
    fn axis_count(&self) -> usize {
        self.axis_count
    }

    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> Result<(), RuntimeError> {
        self.worker
            .request(|reply| HwRequest::MoveSteps {
                steps: steps.to_vec(),
                delays: delays_s.to_vec(),
                reply,
            })
            .map_err(RuntimeError::Backend)?;
        self.sync_positions();
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.positions_cache
    }

    fn read_feedback_steps(&mut self) -> Result<Vec<i64>, RuntimeError> {
        let fb = self
            .worker
            .request(|reply| HwRequest::ReadFeedback { reply })
            .map_err(RuntimeError::Backend)?;
        self.positions_cache = fb.clone();
        Ok(fb)
    }

    fn estop(&mut self) -> Result<(), RuntimeError> {
        self.worker
            .request(|reply| HwRequest::Estop { reply })
            .map_err(RuntimeError::Backend)
    }

    fn home_hardware(&mut self) -> Result<(), RuntimeError> {
        self.worker
            .request(|reply| HwRequest::Home { reply })
            .map_err(RuntimeError::Backend)?;
        self.sync_positions();
        Ok(())
    }
}

fn open_transport(device: &str, baud: u32) -> HwResult<Box<dyn Transport>> {
    if device.eq_ignore_ascii_case("mock") {
        return Ok(Box::new(spyder_runtime::MockTransport::new()));
    }
    if device.contains(':') && !device.starts_with('/') && !device.starts_with("COM") {
        Ok(Box::new(
            TcpTransport::connect(device).map_err(|e| e.to_string())?,
        ))
    } else {
        Ok(Box::new(
            SerialTransport::open(device, baud).map_err(|e| e.to_string())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn channel_backend_is_send_sync() {
        assert_send_sync::<ChannelMotorBackend>();
    }

    #[test]
    fn mock_channel_moves_steps() {
        let mut backend = ChannelMotorBackend::mock(4).unwrap();
        assert_eq!(backend.axis_count(), 4);
        backend
            .move_steps(&[10, -5, 0, 3], &[0.01, 0.01, 0.0, 0.01])
            .unwrap();
        assert_eq!(backend.positions(), &[10, -5, 0, 3]);
    }
}
