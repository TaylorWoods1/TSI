//! Run session and mock playback helpers.

use spyder_core::{Pose, Robot, Vec3};
use spyder_runtime::{
    uniform_axes, MockBackend, MotorBackend, Player, RuntimeError, SafetyLimits,
};

use crate::dto::{PlayLineRequest, PlayLineResponse, RunStatusResponse};

/// In-process run session (mock backend for MVP).
pub struct RunSession {
    /// Backend identifier (`mock`, `stepper`, etc.).
    pub backend_name: String,
    /// Winch/motor axes aligned with cables.
    pub axes: Vec<spyder_runtime::Axis>,
    /// Mock motor backend.
    pub mock: MockBackend,
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
}

impl RunSession {
    /// Connect a mock backend for the given robot.
    pub fn connect_mock(robot: &Robot, home: Vec3) -> Result<Self, String> {
        let n = robot.anchors.len();
        let axes = uniform_axes(n, 0.05, 200.0).map_err(|e| e.to_string())?;
        Ok(Self {
            backend_name: "mock".into(),
            axes,
            mock: MockBackend::new(n),
            home,
            estopped: false,
            last_steps: vec![0; n],
            closed_loop: false,
            realtime: false,
        })
    }

    /// Trip e-stop on mock backend.
    pub fn estop(&mut self) -> Result<(), String> {
        self.estopped = true;
        self.mock.estop().map_err(|e| e.to_string())
    }

    /// Clear software e-stop latch.
    pub fn clear_estop(&mut self) {
        self.estopped = false;
        // Player safety is reconstructed on each play; mock estopped flag cleared on next connect
        self.mock.estopped = false;
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
        let backend = std::mem::replace(&mut self.mock, MockBackend::new(robot.anchors.len()));

        let mut player = Player::new(robot, self.axes.clone(), backend, self.home)
            .map_err(|e| e.to_string())?
            .with_closed_loop(req.closed_loop)
            .with_realtime(req.realtime)
            .with_safety(default_safety());

        let result = player.move_line(start, end, req.segments, 2.0);
        self.mock = player.backend;
        result.map_err(|e| match e {
            RuntimeError::Safety(s) if s.contains("e-stop") => {
                self.estopped = true;
                s
            }
            other => other.to_string(),
        })?;

        self.last_steps = self.mock.positions().to_vec();
        let feedback_pose = pose_from_mock(robot, &self.axes, &self.mock, self.home)?;

        Ok(PlayLineResponse {
            final_steps: self.last_steps.clone(),
            feedback_pose: Some([feedback_pose.x, feedback_pose.y, feedback_pose.z]),
        })
    }

    /// Status snapshot for polling.
    pub fn status(&self, robot: &Robot) -> RunStatusResponse {
        let pose = pose_from_mock(robot, &self.axes, &self.mock, self.home)
            .ok()
            .map(|p| [p.x, p.y, p.z]);
        RunStatusResponse {
            connected: true,
            backend: Some(self.backend_name.clone()),
            estopped: self.estopped,
            steps: Some(self.last_steps.clone()),
            pose,
        }
    }
}

fn default_safety() -> SafetyLimits {
    SafetyLimits {
        min: Vec3::new(-8.0, -8.0, 0.2),
        max: Vec3::new(8.0, 8.0, 7.5),
        max_speed_mps: 2.0,
        ..SafetyLimits::default()
    }
}

fn pose_from_mock(
    robot: &Robot,
    axes: &[spyder_runtime::Axis],
    mock: &MockBackend,
    seed: Vec3,
) -> Result<Vec3, String> {
    let home_ik = robot
        .ik(&Pose::from_position(seed))
        .map_err(|e| e.to_string())?;
    spyder_runtime::pose_from_steps(
        robot,
        &home_ik.lengths,
        &mock.positions(),
        axes,
        seed,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Preset;

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
