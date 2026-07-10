//! Run session and hardware playback helpers.

use spyder_core::{Pose, Robot, Vec3};
use spyder_runtime::{
    uniform_axes, Axis, AxisMap, MockBackend, MotorBackend, Player, RuntimeError, SafetyLimits,
    StepperBackend, TcpTransport, Transport,
};

use crate::dto::{
    ConnectRequest, PlayLineRequest, PlayLineResponse, RunStatusResponse, SafetyLimitsDto,
};

/// Connected motor backend variants (Send-safe for Axum state).
pub enum RunBackend {
    /// In-process mock.
    Mock(MockBackend),
    /// Stepper over TCP (`host:port`).
    Stepper(StepperBackend),
}

impl MotorBackend for RunBackend {
    fn axis_count(&self) -> usize {
        match self {
            RunBackend::Mock(b) => b.axis_count(),
            RunBackend::Stepper(b) => b.axis_count(),
        }
    }

    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> spyder_runtime::Result<()> {
        match self {
            RunBackend::Mock(b) => b.move_steps(steps, delays_s),
            RunBackend::Stepper(b) => b.move_steps(steps, delays_s),
        }
    }

    fn positions(&self) -> &[i64] {
        match self {
            RunBackend::Mock(b) => b.positions(),
            RunBackend::Stepper(b) => b.positions(),
        }
    }

    fn read_feedback_steps(&mut self) -> spyder_runtime::Result<Vec<i64>> {
        match self {
            RunBackend::Mock(b) => b.read_feedback_steps(),
            RunBackend::Stepper(b) => b.read_feedback_steps(),
        }
    }

    fn estop(&mut self) -> spyder_runtime::Result<()> {
        match self {
            RunBackend::Mock(b) => b.estop(),
            RunBackend::Stepper(b) => b.estop(),
        }
    }

    fn home_hardware(&mut self) -> spyder_runtime::Result<()> {
        match self {
            RunBackend::Mock(b) => b.home_hardware(),
            RunBackend::Stepper(b) => b.home_hardware(),
        }
    }
}

/// In-process run session.
pub struct RunSession {
    /// Backend identifier.
    pub backend_name: String,
    /// Winch/motor axes aligned with cables.
    pub axes: Vec<Axis>,
    /// Connected backend.
    pub backend: RunBackend,
    /// Home pose for playback.
    pub home: Vec3,
    /// Software e-stop latched.
    pub estopped: bool,
    /// Last known step positions.
    pub last_steps: Vec<i64>,
    /// Closed-loop correction toggle.
    pub closed_loop: bool,
    /// Wall-clock realtime playback toggle.
    pub realtime: bool,
    /// Active safety limits.
    pub safety: SafetyLimits,
}

impl RunSession {
    /// Connect using request parameters.
    pub fn connect(robot: &Robot, home: Vec3, req: &ConnectRequest) -> Result<Self, String> {
        let n = robot.anchors.len();
        let axes = uniform_axes(n, 0.05, 200.0).map_err(|e| e.to_string())?;
        let safety = default_safety();
        match req.backend.as_str() {
            "mock" => Ok(Self {
                backend_name: "mock".into(),
                axes,
                backend: RunBackend::Mock(MockBackend::new(n)),
                home,
                estopped: false,
                last_steps: vec![0; n],
                closed_loop: false,
                realtime: false,
                safety,
            }),
            "stepper" => {
                let device = req
                    .device
                    .clone()
                    .ok_or_else(|| "stepper requires device host:port".to_string())?;
                if !device.contains(':') {
                    return Err(
                        "GUI stepper backend requires TCP host:port (serial use CLI)".into(),
                    );
                }
                let transport = open_tcp(&device)?;
                Ok(Self {
                    backend_name: "stepper".into(),
                    axes,
                    backend: RunBackend::Stepper(StepperBackend::new(transport, n)),
                    home,
                    estopped: false,
                    last_steps: vec![0; n],
                    closed_loop: false,
                    realtime: false,
                    safety,
                })
            }
            "odrive" => Err(
                "odrive GUI connect: use CLI or TCP stepper sim for now".into(),
            ),
            "multiboard" => {
                let map_val = req
                    .axis_map
                    .clone()
                    .ok_or_else(|| "multiboard requires axis_map JSON".to_string())?;
                let map: AxisMap =
                    serde_json::from_value(map_val).map_err(|e| format!("axis_map: {e}"))?;
                map.validate().map_err(|e| e.to_string())?;
                // Dry-run: mock backend with cable count from axis map.
                Ok(Self {
                    backend_name: "multiboard-mock".into(),
                    axes,
                    backend: RunBackend::Mock(MockBackend::new(map.cables.len())),
                    home,
                    estopped: false,
                    last_steps: vec![0; n],
                    closed_loop: false,
                    realtime: false,
                    safety,
                })
            }
            other => Err(format!("unknown backend: {other}")),
        }
    }

    /// Connect a mock backend for the given robot (tests).
    pub fn connect_mock(robot: &Robot, home: Vec3) -> Result<Self, String> {
        Self::connect(
            robot,
            home,
            &ConnectRequest {
                backend: "mock".into(),
                device: None,
                baud: None,
                axis_map: None,
            },
        )
    }

    /// Trip e-stop.
    pub fn estop(&mut self) -> Result<(), String> {
        self.estopped = true;
        self.backend.estop().map_err(|e| e.to_string())
    }

    /// Clear software e-stop latch.
    pub fn clear_estop(&mut self) {
        self.estopped = false;
        if let RunBackend::Mock(ref mut m) = self.backend {
            m.estopped = false;
        }
    }

    /// Play a straight-line trajectory.
    pub fn play_line(
        &mut self,
        robot: &Robot,
        req: &PlayLineRequest,
    ) -> Result<PlayLineResponse, String> {
        if self.estopped {
            return Err("e-stop latched".into());
        }
        self.closed_loop = req.closed_loop;
        self.realtime = req.realtime;

        let start = Vec3::new(req.start[0], req.start[1], req.start[2]);
        let end = Vec3::new(req.end[0], req.end[1], req.end[2]);
        let n = robot.anchors.len();
        let placeholder = RunBackend::Mock(MockBackend::new(n));
        let backend = std::mem::replace(&mut self.backend, placeholder);

        let mut player = Player::new(robot, self.axes.clone(), backend, self.home)
            .map_err(|e| e.to_string())?
            .with_closed_loop(req.closed_loop)
            .with_realtime(req.realtime)
            .with_safety(self.safety.clone());

        let result = player.move_line(start, end, req.segments, 2.0);
        self.backend = player.backend;
        result.map_err(|e| match e {
            RuntimeError::Safety(s) if s.contains("e-stop") => {
                self.estopped = true;
                s
            }
            other => other.to_string(),
        })?;

        self.last_steps = self.backend.positions().to_vec();
        let feedback_pose = pose_from_backend(robot, &self.axes, &self.backend, self.home)?;

        Ok(PlayLineResponse {
            final_steps: self.last_steps.clone(),
            feedback_pose: Some([feedback_pose.x, feedback_pose.y, feedback_pose.z]),
        })
    }

    /// Status snapshot for polling.
    pub fn status(&self, robot: &Robot) -> RunStatusResponse {
        let pose = pose_from_backend(robot, &self.axes, &self.backend, self.home)
            .ok()
            .map(|p| [p.x, p.y, p.z]);
        RunStatusResponse {
            connected: true,
            backend: Some(self.backend_name.clone()),
            estopped: self.estopped,
            steps: Some(self.last_steps.clone()),
            pose,
            safety: Some(SafetyLimitsDto {
                min: [self.safety.min.x, self.safety.min.y, self.safety.min.z],
                max: [self.safety.max.x, self.safety.max.y, self.safety.max.z],
                max_speed_mps: self.safety.max_speed_mps,
            }),
        }
    }
}

fn open_tcp(device: &str) -> Result<Box<dyn Transport>, String> {
    Ok(Box::new(
        TcpTransport::connect(device).map_err(|e| e.to_string())?,
    ))
}

fn default_safety() -> SafetyLimits {
    SafetyLimits {
        min: Vec3::new(-8.0, -8.0, 0.2),
        max: Vec3::new(8.0, 8.0, 7.5),
        max_speed_mps: 2.0,
        ..SafetyLimits::default()
    }
}

fn pose_from_backend(
    robot: &Robot,
    axes: &[Axis],
    backend: &RunBackend,
    seed: Vec3,
) -> Result<Vec3, String> {
    let home_ik = robot
        .ik(&Pose::from_position(seed))
        .map_err(|e| e.to_string())?;
    let steps: Vec<i64> = backend.positions().to_vec();
    spyder_runtime::pose_from_steps(robot, &home_ik.lengths, &steps, axes, seed)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Preset;

    fn assert_send<T: Send>() {}

    #[test]
    fn backends_are_send() {
        assert_send::<MockBackend>();
        assert_send::<RunBackend>();
        assert_send::<RunSession>();
    }

    #[test]
    fn mock_play_returns_nonzero_steps() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let home = Vec3::new(0.0, 0.0, 1.5);
        let mut session = RunSession::connect_mock(&robot, home).unwrap();
        let resp = session
            .play_line(
                &robot,
                &PlayLineRequest {
                    start: [0.0, 0.0, 1.5],
                    end: [0.5, 0.0, 1.5],
                    segments: 5,
                    closed_loop: false,
                    realtime: false,
                },
            )
            .unwrap();
        assert!(resp.final_steps.iter().any(|s| *s != 0));
    }

    #[test]
    fn estop_blocks_play_until_cleared() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let home = Vec3::new(0.0, 0.0, 1.5);
        let mut session = RunSession::connect_mock(&robot, home).unwrap();
        session.estop().unwrap();
        let err = session.play_line(
            &robot,
            &PlayLineRequest {
                start: [0.0, 0.0, 1.5],
                end: [0.5, 0.0, 1.5],
                segments: 3,
                closed_loop: false,
                realtime: false,
            },
        );
        assert!(err.is_err());
        session.clear_estop();
        assert!(session
            .play_line(
                &robot,
                &PlayLineRequest {
                    start: [0.0, 0.0, 1.5],
                    end: [0.3, 0.0, 1.5],
                    segments: 3,
                    closed_loop: false,
                    realtime: false,
                },
            )
            .is_ok());
    }
}
