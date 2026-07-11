//! Run session and hardware playback helpers.

use spyder_core::{Pose, Robot, Vec3};
use spyder_runtime::{
    uniform_axes, Axis, AxisMap, Calibration, MockBackend, MotorBackend, Player, RuntimeError,
    SafetyLimits,
};

use crate::dto::{
    ConnectRequest, MotorAxisDto, PlayLineRequest, PlayLineResponse, PlayWaypointsRequest,
    PlayWaypointsResponse, RunStatusResponse, SafetyLimitsDto,
};
use crate::hw_thread::ChannelMotorBackend;

/// Connected motor backend variants (Send-safe for Axum state).
pub enum RunBackend {
    /// In-process mock.
    Mock(MockBackend),
    /// Hardware proxy (stepper, odrive, multiboard).
    Channel(ChannelMotorBackend),
}

impl MotorBackend for RunBackend {
    fn axis_count(&self) -> usize {
        match self {
            RunBackend::Mock(b) => b.axis_count(),
            RunBackend::Channel(b) => b.axis_count(),
        }
    }

    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> spyder_runtime::Result<()> {
        match self {
            RunBackend::Mock(b) => b.move_steps(steps, delays_s),
            RunBackend::Channel(b) => b.move_steps(steps, delays_s),
        }
    }

    fn positions(&self) -> &[i64] {
        match self {
            RunBackend::Mock(b) => b.positions(),
            RunBackend::Channel(b) => b.positions(),
        }
    }

    fn read_feedback_steps(&mut self) -> spyder_runtime::Result<Vec<i64>> {
        match self {
            RunBackend::Mock(b) => b.read_feedback_steps(),
            RunBackend::Channel(b) => b.read_feedback_steps(),
        }
    }

    fn estop(&mut self) -> spyder_runtime::Result<()> {
        match self {
            RunBackend::Mock(b) => b.estop(),
            RunBackend::Channel(b) => b.estop(),
        }
    }

    fn home_hardware(&mut self) -> spyder_runtime::Result<()> {
        match self {
            RunBackend::Mock(b) => b.home_hardware(),
            RunBackend::Channel(b) => b.home_hardware(),
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
    /// Reference lengths at home (from calibration when present).
    pub home_lengths: Vec<f64>,
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
    pub fn connect(
        robot: &Robot,
        home: Vec3,
        req: &ConnectRequest,
        motor_axes: &[MotorAxisDto],
        calibration: Option<&Calibration>,
    ) -> Result<Self, String> {
        let n = robot.anchors.len();
        let axes = axes_from_motor_dtos(motor_axes, n)?;
        let home_lengths = home_lengths_for(robot, home, calibration)?;
        let safety = default_safety();
        let baud = req.baud.unwrap_or(115_200);

        let (backend_name, backend) = match req.backend.as_str() {
            "mock" => ("mock".into(), RunBackend::Mock(MockBackend::new(n))),
            "stepper" => {
                let device = req
                    .device
                    .clone()
                    .ok_or_else(|| "stepper requires device path or host:port".to_string())?;
                (
                    "stepper".into(),
                    RunBackend::Channel(ChannelMotorBackend::stepper(&device, baud, n)?),
                )
            }
            "odrive" => {
                let device = req
                    .device
                    .clone()
                    .ok_or_else(|| "odrive requires device path or host:port".to_string())?;
                (
                    "odrive".into(),
                    RunBackend::Channel(ChannelMotorBackend::odrive(&device, baud, n)?),
                )
            }
            "multiboard" => {
                let map_val = req
                    .axis_map
                    .clone()
                    .ok_or_else(|| "multiboard requires axis_map JSON".to_string())?;
                let map: AxisMap =
                    serde_json::from_value(map_val).map_err(|e| format!("axis_map: {e}"))?;
                map.validate().map_err(|e| e.to_string())?;
                let name = if req.mock {
                    "multiboard-mock"
                } else {
                    "multiboard"
                };
                (
                    name.into(),
                    RunBackend::Channel(ChannelMotorBackend::multiboard(map, req.mock)?),
                )
            }
            other => return Err(format!("unknown backend: {other}")),
        };

        let last_steps = vec![0; n];
        Ok(Self {
            backend_name,
            axes,
            backend,
            home,
            home_lengths,
            estopped: false,
            last_steps,
            closed_loop: false,
            realtime: false,
            safety,
        })
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
                mock: false,
            },
            &[],
            None,
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

        let mut player = make_player(
            robot,
            self.axes.clone(),
            &self.home_lengths,
            backend,
            start,
        )?
            .with_closed_loop(req.closed_loop)
            .with_realtime(req.realtime)
            .with_safety(self.safety.clone());

        let result = player.move_line(start, end, req.segments, 2.0);
        self.backend = player.backend;
        result.map_err(|e| map_safety_error(e, self))?;

        self.last_steps = self.backend.positions().to_vec();
        let feedback_pose = pose_from_backend(robot, &self.axes, &self.backend, self.home, &self.home_lengths)?;

        Ok(PlayLineResponse {
            final_steps: self.last_steps.clone(),
            feedback_pose: Some([feedback_pose.x, feedback_pose.y, feedback_pose.z]),
        })
    }

    /// Play through a list of Cartesian waypoints.
    pub fn play_waypoints(
        &mut self,
        robot: &Robot,
        req: &PlayWaypointsRequest,
    ) -> Result<PlayWaypointsResponse, String> {
        if self.estopped {
            return Err("e-stop latched".into());
        }
        if req.waypoints.len() < 2 {
            return Err("need at least 2 waypoints".into());
        }
        self.closed_loop = req.closed_loop;
        self.realtime = req.realtime;

        let n = robot.anchors.len();
        let placeholder = RunBackend::Mock(MockBackend::new(n));
        let backend = std::mem::replace(&mut self.backend, placeholder);

        let start = Vec3::new(
            req.waypoints[0][0],
            req.waypoints[0][1],
            req.waypoints[0][2],
        );
        let mut player = make_player(
            robot,
            self.axes.clone(),
            &self.home_lengths,
            backend,
            start,
        )?
            .with_closed_loop(req.closed_loop)
            .with_realtime(req.realtime)
            .with_safety(self.safety.clone());

        let per = req.duration_s / (req.waypoints.len() - 1).max(1) as f64;
        for wp in req.waypoints.iter().skip(1) {
            let target = Vec3::new(wp[0], wp[1], wp[2]);
            match player.move_to(target, per) {
                Ok(_) => {}
                Err(RuntimeError::Safety(s)) if s.contains("e-stop") => {
                    self.estopped = true;
                    self.backend = player.backend;
                    return Err(s);
                }
                Err(e) => {
                    self.backend = player.backend;
                    return Err(e.to_string());
                }
            }
        }

        self.backend = player.backend;
        self.last_steps = self.backend.positions().to_vec();
        let feedback_pose = pose_from_backend(robot, &self.axes, &self.backend, self.home, &self.home_lengths)?;

        Ok(PlayWaypointsResponse {
            final_steps: self.last_steps.clone(),
            feedback_pose: Some([feedback_pose.x, feedback_pose.y, feedback_pose.z]),
        })
    }

    /// Status snapshot for polling.
    pub fn status(&self, robot: &Robot) -> RunStatusResponse {
        let pose = pose_from_backend(robot, &self.axes, &self.backend, self.home, &self.home_lengths)
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

fn make_player<'a>(
    robot: &'a Robot,
    axes: Vec<Axis>,
    home_lengths: &[f64],
    backend: RunBackend,
    start: Vec3,
) -> Result<Player<'a, RunBackend>, String> {
    let mut player = Player::new(robot, axes, backend, start).map_err(|e| e.to_string())?;
    if home_lengths.len() == player.axes.len() {
        player.home_lengths = home_lengths.to_vec();
        player.current_lengths = home_lengths.to_vec();
    }
    Ok(player)
}

fn map_safety_error(e: RuntimeError, session: &mut RunSession) -> String {
    match e {
        RuntimeError::Safety(s) if s.contains("e-stop") => {
            session.estopped = true;
            s
        }
        other => other.to_string(),
    }
}

fn axes_from_motor_dtos(dtos: &[MotorAxisDto], n: usize) -> Result<Vec<Axis>, String> {
    if dtos.len() == n && !dtos.is_empty() {
        let mut axes = Vec::with_capacity(n);
        for dto in dtos {
            axes.push(
                Axis::new(dto.drum_radius_m, dto.steps_per_rev, 1.0)
                    .map_err(|e| e.to_string())?,
            );
        }
        return Ok(axes);
    }
    uniform_axes(n, 0.05, 200.0).map_err(|e| e.to_string())
}

fn home_lengths_for(
    robot: &Robot,
    home: Vec3,
    calibration: Option<&Calibration>,
) -> Result<Vec<f64>, String> {
    if let Some(cal) = calibration {
        if cal.home_lengths_m.len() == robot.anchors.len() {
            return Ok(cal.home_lengths_m.clone());
        }
    }
    let ik = robot
        .ik(&Pose::from_position(home))
        .map_err(|e| e.to_string())?;
    Ok(ik.lengths)
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
    home_lengths: &[f64],
) -> Result<Vec3, String> {
    let steps: Vec<i64> = backend.positions().to_vec();
    spyder_runtime::pose_from_steps(robot, home_lengths, &steps, axes, seed)
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
        assert_send::<ChannelMotorBackend>();
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
    fn play_waypoints_moves() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let home = Vec3::new(0.0, 0.0, 1.5);
        let mut session = RunSession::connect_mock(&robot, home).unwrap();
        let resp = session
            .play_waypoints(
                &robot,
                &PlayWaypointsRequest {
                    waypoints: vec![
                        [0.0, 0.0, 1.5],
                        [0.3, 0.0, 1.5],
                        [0.3, 0.2, 1.5],
                    ],
                    duration_s: 2.0,
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
