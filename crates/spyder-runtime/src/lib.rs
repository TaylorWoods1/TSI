//! Motor backends and trajectory playback for Spyder.
//!
//! Wraps `spyder-actuation` with hardware I/O: mock, TCP stepper, and ODrive
//! backends, plus calibration, axis maps, closed-loop feedback, and safety limits.

#![deny(missing_docs)]

mod axis_map;
mod calibration;
mod feedback;
mod multi_board;
mod odrive;
mod safety;
mod stepper;
mod transport;

use spyder_actuation::{length_delta_to_command, synchronized_step_delays, Motor, MotorCommand, Winch};
use spyder_core::{Pose, Robot, Vec3};
use spyder_sim::{line_waypoints, trajectory_lengths};
use thiserror::Error;

pub use axis_map::{AxisEndpoint, AxisMap};
pub use calibration::{apply_anchor_override, venue_toml_from_anchors, Calibration};
pub use feedback::{length_error, lengths_from_steps, pose_from_steps, uniform_axes};
pub use multi_board::MultiBoardBackend;
pub use odrive::{ODriveAxis, ODriveBackend};
pub use safety::{SafetyError, SafetyLimits};
pub use stepper::StepperBackend;
pub use transport::{MockTransport, SerialTransport, TcpTransport, Transport};

/// Runtime errors.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Kinematics / config failure.
    #[error(transparent)]
    Core(#[from] spyder_core::SpyderError),
    /// Backend / transport failure.
    #[error("backend: {0}")]
    Backend(String),
    /// Bad arguments.
    #[error("{0}")]
    Config(String),
    /// Soft-limit / e-stop / slack violation.
    #[error("safety: {0}")]
    Safety(String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Low-level motor interface.
pub trait MotorBackend {
    /// Number of axes.
    fn axis_count(&self) -> usize;

    /// Simultaneous step move.
    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> Result<()>;

    /// Bookkeeping positions (host-side cumulative steps).
    fn positions(&self) -> &[i64];

    /// Optional hardware feedback positions (steps). Default: clone bookkeeping.
    fn read_feedback_steps(&mut self) -> Result<Vec<i64>> {
        Ok(self.positions().to_vec())
    }

    /// Optional e-stop / disable drivers. Default: no-op.
    fn estop(&mut self) -> Result<()> {
        Ok(())
    }

    /// Optional home/zero on hardware. Default: no-op.
    fn home_hardware(&mut self) -> Result<()> {
        Ok(())
    }
}

/// In-memory mock backend.
#[derive(Clone, Debug)]
pub struct MockBackend {
    /// Cumulative steps.
    pub steps: Vec<i64>,
    /// Command log.
    pub log: Vec<Vec<i64>>,
    /// Simulated feedback (= steps unless injected error).
    pub feedback: Vec<i64>,
    /// Whether estop was called.
    pub estopped: bool,
}

impl MockBackend {
    /// Create mock with `n` axes.
    pub fn new(n: usize) -> Self {
        Self {
            steps: vec![0; n],
            log: Vec::new(),
            feedback: vec![0; n],
            estopped: false,
        }
    }
}

impl MotorBackend for MockBackend {
    fn axis_count(&self) -> usize {
        self.steps.len()
    }

    fn move_steps(&mut self, steps: &[i64], _delays_s: &[f64]) -> Result<()> {
        if steps.len() != self.steps.len() {
            return Err(RuntimeError::Config("step vector length mismatch".into()));
        }
        for (i, &step) in steps.iter().enumerate() {
            self.steps[i] += step;
            self.feedback[i] = self.steps[i];
        }
        self.log.push(steps.to_vec());
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.steps
    }

    fn read_feedback_steps(&mut self) -> Result<Vec<i64>> {
        Ok(self.feedback.clone())
    }

    fn estop(&mut self) -> Result<()> {
        self.estopped = true;
        Ok(())
    }

    fn home_hardware(&mut self) -> Result<()> {
        for s in &mut self.steps {
            *s = 0;
        }
        for f in &mut self.feedback {
            *f = 0;
        }
        Ok(())
    }
}

/// Winch + motor pair.
#[derive(Clone, Debug)]
pub struct Axis {
    /// Winch.
    pub winch: Winch,
    /// Motor.
    pub motor: Motor,
}

impl Axis {
    /// Construct axis.
    pub fn new(radius_m: f64, steps_per_rev: f64, gear_ratio: f64) -> Result<Self> {
        Ok(Self {
            winch: Winch::new(radius_m, 1.0).map_err(|e| RuntimeError::Config(e.to_string()))?,
            motor: Motor::new(steps_per_rev, gear_ratio)
                .map_err(|e| RuntimeError::Config(e.to_string()))?,
        })
    }
}

/// Trajectory player with safety + optional closed-loop correction.
pub struct Player<'a, B: MotorBackend> {
    /// Robot.
    pub robot: &'a Robot,
    /// Axes.
    pub axes: Vec<Axis>,
    /// Backend.
    pub backend: B,
    /// Current commanded lengths.
    pub current_lengths: Vec<f64>,
    /// Home lengths (calibration zero).
    pub home_lengths: Vec<f64>,
    /// Last known Cartesian seed for FK.
    pub pose_seed: Vec3,
    /// Safety policy.
    pub safety: SafetyLimits,
    /// If true, after each move read feedback and apply one corrective step.
    pub closed_loop: bool,
    /// If true, sleep for each segment duration (wall-clock realtime playback).
    pub realtime: bool,
}

impl<'a, B: MotorBackend> Player<'a, B> {
    /// Construct player at `home` with default safety.
    pub fn new(robot: &'a Robot, axes: Vec<Axis>, backend: B, home: Vec3) -> Result<Self> {
        if axes.len() != robot.anchors.len() {
            return Err(RuntimeError::Config(
                "axes count must match cable count".into(),
            ));
        }
        if backend.axis_count() != axes.len() {
            return Err(RuntimeError::Config(
                "backend axis_count must match axes".into(),
            ));
        }
        let ik = robot.ik(&Pose::from_position(home))?;
        Ok(Self {
            robot,
            axes,
            backend,
            current_lengths: ik.lengths.clone(),
            home_lengths: ik.lengths,
            pose_seed: home,
            safety: SafetyLimits::default(),
            closed_loop: false,
            realtime: false,
        })
    }

    /// Enable closed-loop correction.
    pub fn with_closed_loop(mut self, on: bool) -> Self {
        self.closed_loop = on;
        self
    }

    /// Enable wall-clock sleeps matching segment durations.
    pub fn with_realtime(mut self, on: bool) -> Self {
        self.realtime = on;
        self
    }

    /// Replace safety limits.
    pub fn with_safety(mut self, safety: SafetyLimits) -> Self {
        self.safety = safety;
        self
    }

    /// Load home lengths from calibration.
    pub fn apply_calibration(&mut self, cal: &Calibration) -> Result<()> {
        if cal.home_lengths_m.len() != self.axes.len() {
            return Err(RuntimeError::Config(
                "calibration cable count mismatch".into(),
            ));
        }
        self.home_lengths = cal.home_lengths_m.clone();
        self.current_lengths = cal.home_lengths_m.clone();
        self.pose_seed = cal.home_vec();
        Ok(())
    }

    /// Hardware + bookkeeping home.
    pub fn home(&mut self) -> Result<()> {
        self.safety.check_estop()?;
        self.backend.home_hardware()?;
        self.current_lengths = self.home_lengths.clone();
        Ok(())
    }

    /// Trip software + hardware e-stop.
    pub fn estop(&mut self) -> Result<()> {
        self.safety.trip_estop();
        self.backend.estop()
    }

    /// Clear software e-stop only.
    pub fn clear_estop(&mut self) {
        self.safety.clear_estop();
    }

    /// FK pose from backend feedback.
    pub fn feedback_pose(&mut self) -> Result<Vec3> {
        let steps = self.backend.read_feedback_steps()?;
        pose_from_steps(
            self.robot,
            &self.home_lengths,
            &steps,
            &self.axes,
            self.pose_seed,
        )
    }

    fn issue_length_move(&mut self, target_lengths: &[f64], duration_s: f64) -> Result<Vec<MotorCommand>> {
        self.safety.check_lengths(target_lengths)?;
        let mut cmds = Vec::with_capacity(target_lengths.len());
        let mut steps = Vec::with_capacity(target_lengths.len());
        for (i, &target) in target_lengths.iter().enumerate() {
            let delta = target - self.current_lengths[i];
            let cmd = length_delta_to_command(&self.axes[i].winch, &self.axes[i].motor, delta);
            steps.push(cmd.steps);
            cmds.push(cmd);
        }
        self.safety.check_steps(&steps)?;
        let delays = synchronized_step_delays(&steps, duration_s);
        self.backend.move_steps(&steps, &delays)?;
        self.current_lengths = target_lengths.to_vec();
        Ok(cmds)
    }

    /// Move to Cartesian target with safety checks.
    pub fn move_to(&mut self, target: Vec3, duration_s: f64) -> Result<Vec<MotorCommand>> {
        self.safety.check_pose(&target)?;
        self.safety
            .check_segment(&self.pose_seed, &target, duration_s)?;
        let ik = self.robot.ik(&Pose::from_position(target))?;
        let cmds = self.issue_length_move(&ik.lengths, duration_s)?;
        self.pose_seed = target;

        if self.closed_loop {
            let measured = lengths_from_steps(
                &self.home_lengths,
                &self.backend.read_feedback_steps()?,
                &self.axes,
            )?;
            let err = length_error(&ik.lengths, &measured)?;
            let max_err = err.iter().map(|e| e.abs()).fold(0.0, f64::max);
            if max_err > 1e-4 {
                // One corrective move toward commanded lengths (short duration).
                let mut corrected = measured.clone();
                for i in 0..corrected.len() {
                    corrected[i] += err[i];
                }
                let _ = self.issue_length_move(&corrected, (duration_s * 0.25).max(0.05))?;
            }
            if let Ok(p) = self.feedback_pose() {
                self.pose_seed = p;
            }
        }
        if self.realtime && duration_s > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f64(duration_s));
        }
        Ok(cmds)
    }

    /// Straight line with per-segment safety.
    pub fn move_line(
        &mut self,
        start: Vec3,
        end: Vec3,
        segments: usize,
        duration_s: f64,
    ) -> Result<()> {
        self.safety.check_pose(&start)?;
        self.safety.check_pose(&end)?;
        let pts = line_waypoints(start, end, segments);
        let start_ik = self.robot.ik(&Pose::from_position(start))?;
        self.current_lengths = start_ik.lengths;
        self.pose_seed = start;
        let per = duration_s / segments.max(1) as f64;
        for p in pts.iter().skip(1) {
            self.move_to(*p, per)?;
        }
        Ok(())
    }

    /// Plan lengths only.
    pub fn plan_line_lengths(
        &self,
        start: Vec3,
        end: Vec3,
        segments: usize,
    ) -> Result<Vec<Vec<f64>>> {
        let pts = line_waypoints(start, end, segments);
        Ok(trajectory_lengths(self.robot, &pts)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Preset;

    #[test]
    fn mock_player_line_accumulates_steps() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let axes: Vec<_> = (0..4)
            .map(|_| Axis::new(0.05, 200.0, 1.0).unwrap())
            .collect();
        let backend = MockBackend::new(4);
        let home = Vec3::new(0.0, 0.0, 1.5);
        let mut player = Player::new(&robot, axes, backend, home)
            .unwrap()
            .with_safety(SafetyLimits {
                min: Vec3::new(-2.0, -2.0, 0.2),
                max: Vec3::new(2.0, 2.0, 3.0),
                max_speed_mps: 2.0,
                ..SafetyLimits::default()
            });
        player
            .move_line(home, Vec3::new(0.5, 0.0, 1.5), 5, 2.0)
            .unwrap();
        assert!(player.backend.positions().iter().any(|s| *s != 0));
    }

    #[test]
    fn safety_blocks_out_of_bounds() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let axes = uniform_axes(4, 0.05, 200.0).unwrap();
        let mut player = Player::new(&robot, axes, MockBackend::new(4), Vec3::new(0.0, 0.0, 1.0))
            .unwrap()
            .with_safety(SafetyLimits {
                max: Vec3::new(0.2, 0.2, 2.0),
                ..SafetyLimits::default()
            });
        let err = player.move_to(Vec3::new(1.0, 0.0, 1.0), 2.0);
        assert!(err.is_err());
    }

    #[test]
    fn estop_latches() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let axes = uniform_axes(4, 0.05, 200.0).unwrap();
        let mut player =
            Player::new(&robot, axes, MockBackend::new(4), Vec3::new(0.0, 0.0, 1.0)).unwrap();
        player.estop().unwrap();
        assert!(player.backend.estopped);
        assert!(player.move_to(Vec3::new(0.1, 0.0, 1.0), 1.0).is_err());
        player.clear_estop();
        assert!(player.move_to(Vec3::new(0.1, 0.0, 1.0), 1.0).is_ok());
    }

    #[test]
    fn plan_line_lengths_matches_waypoint_count() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let axes = uniform_axes(4, 0.05, 200.0).unwrap();
        let home = Vec3::new(0.0, 0.0, 1.0);
        let player = Player::new(&robot, axes, MockBackend::new(4), home).unwrap();
        let lengths = player
            .plan_line_lengths(home, Vec3::new(0.3, 0.0, 1.0), 4)
            .unwrap();
        assert_eq!(lengths.len(), 5);
    }

    #[test]
    fn apply_calibration_updates_home_lengths() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let axes = uniform_axes(4, 0.05, 200.0).unwrap();
        let home = Vec3::new(0.0, 0.0, 1.0);
        let mut player = Player::new(&robot, axes, MockBackend::new(4), home).unwrap();
        let cal = Calibration::capture(&robot, home, 0.05, 200.0).unwrap();
        player.apply_calibration(&cal).unwrap();
        assert_eq!(player.home_lengths, cal.home_lengths_m);
    }
}
