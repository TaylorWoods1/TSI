//! Motor backend trait and playback engine.

#![deny(missing_docs)]

use spyder_actuation::{length_delta_to_command, synchronized_step_delays, Motor, MotorCommand, Winch};
use spyder_core::{Pose, Robot, Vec3};
use spyder_sim::{line_waypoints, trajectory_lengths};
use thiserror::Error;

/// Runtime errors.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Kinematics / config failure.
    #[error(transparent)]
    Core(#[from] spyder_core::SpyderError),
    /// Backend failure.
    #[error("backend: {0}")]
    Backend(String),
    /// Bad arguments.
    #[error("{0}")]
    Config(String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Low-level motor interface implemented by mock, stepper, ODrive, etc.
pub trait MotorBackend {
    /// Number of axes this backend drives.
    fn axis_count(&self) -> usize;

    /// Execute a simultaneous step move. `steps[i]` may be negative.
    /// `delays_s[i]` is seconds between steps for axis i (0 = idle).
    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> Result<()>;

    /// Optional: report cumulative steps since construction.
    fn positions(&self) -> &[i64];
}

/// In-memory mock backend for tests and dry-runs.
#[derive(Clone, Debug)]
pub struct MockBackend {
    /// Cumulative step counts per axis.
    pub steps: Vec<i64>,
    /// Log of commanded moves.
    pub log: Vec<Vec<i64>>,
}

impl MockBackend {
    /// Create a mock with `n` axes at zero.
    pub fn new(n: usize) -> Self {
        Self {
            steps: vec![0; n],
            log: Vec::new(),
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
        for (acc, d) in self.steps.iter_mut().zip(steps.iter()) {
            *acc += *d;
        }
        self.log.push(steps.to_vec());
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.steps
    }
}

/// Placeholder stepper backend (logs intent; no GPIO yet).
#[derive(Clone, Debug)]
pub struct StepperStub {
    /// Cumulative steps.
    pub steps: Vec<i64>,
    /// Human-readable pulse log.
    pub pulses: Vec<String>,
}

impl StepperStub {
    /// Create stub for `n` steppers.
    pub fn new(n: usize) -> Self {
        Self {
            steps: vec![0; n],
            pulses: Vec::new(),
        }
    }
}

impl MotorBackend for StepperStub {
    fn axis_count(&self) -> usize {
        self.steps.len()
    }

    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> Result<()> {
        if steps.len() != self.steps.len() {
            return Err(RuntimeError::Config("step vector length mismatch".into()));
        }
        for (i, (d, delay)) in steps.iter().zip(delays_s.iter()).enumerate() {
            self.pulses.push(format!(
                "axis{i}: steps={d} delay_s={delay:.6}"
            ));
            self.steps[i] += *d;
        }
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.steps
    }
}

/// Winch + motor pair for one cable.
#[derive(Clone, Debug)]
pub struct Axis {
    /// Winch geometry.
    pub winch: Winch,
    /// Motor parameters.
    pub motor: Motor,
}

impl Axis {
    /// Convenience constructor.
    pub fn new(radius_m: f64, steps_per_rev: f64, gear_ratio: f64) -> Result<Self> {
        Ok(Self {
            winch: Winch::new(radius_m, 1.0).map_err(|e| RuntimeError::Config(e.to_string()))?,
            motor: Motor::new(steps_per_rev, gear_ratio)
                .map_err(|e| RuntimeError::Config(e.to_string()))?,
        })
    }
}

/// Plays Cartesian trajectories by IK → length deltas → synchronized steps.
pub struct Player<'a, B: MotorBackend> {
    /// Robot kinematics.
    pub robot: &'a Robot,
    /// Per-cable axes.
    pub axes: Vec<Axis>,
    /// Motor backend.
    pub backend: B,
    /// Last commanded cable lengths (home / current).
    pub current_lengths: Vec<f64>,
}

impl<'a, B: MotorBackend> Player<'a, B> {
    /// Construct a player; `home` sets the initial length reference via IK.
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
            current_lengths: ik.lengths,
        })
    }

    /// Move to a Cartesian point over `duration_s` (synchronized winches).
    pub fn move_to(&mut self, target: Vec3, duration_s: f64) -> Result<Vec<MotorCommand>> {
        let ik = self.robot.ik(&Pose::from_position(target))?;
        let mut cmds = Vec::with_capacity(ik.lengths.len());
        let mut steps = Vec::with_capacity(ik.lengths.len());
        for i in 0..ik.lengths.len() {
            let delta = ik.lengths[i] - self.current_lengths[i];
            let cmd = length_delta_to_command(&self.axes[i].winch, &self.axes[i].motor, delta);
            steps.push(cmd.steps);
            cmds.push(cmd);
        }
        let delays = synchronized_step_delays(&steps, duration_s);
        self.backend.move_steps(&steps, &delays)?;
        self.current_lengths = ik.lengths;
        Ok(cmds)
    }

    /// Follow a straight line from current IK pose estimate is not tracked in Cartesian;
    /// provide explicit `start` and `end` with `segments` intermediate IK samples.
    pub fn move_line(
        &mut self,
        start: Vec3,
        end: Vec3,
        segments: usize,
        duration_s: f64,
    ) -> Result<()> {
        let pts = line_waypoints(start, end, segments);
        // Re-home length reference at start.
        let start_ik = self.robot.ik(&Pose::from_position(start))?;
        self.current_lengths = start_ik.lengths;
        let per = duration_s / segments.max(1) as f64;
        for p in pts.iter().skip(1) {
            self.move_to(*p, per)?;
        }
        Ok(())
    }

    /// Precompute length schedule for a polyline (no motion).
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
        let mut player = Player::new(&robot, axes, backend, home).unwrap();
        player
            .move_line(home, Vec3::new(0.5, 0.0, 1.5), 5, 1.0)
            .unwrap();
        assert!(!player.backend.log.is_empty());
        assert_eq!(player.backend.positions().len(), 4);
        // Not all zeros after a real move
        assert!(player.backend.positions().iter().any(|s| *s != 0));
    }

    #[test]
    fn stepper_stub_records_pulses() {
        let mut stub = StepperStub::new(3);
        stub.move_steps(&[10, -5, 0], &[0.01, 0.02, 0.0]).unwrap();
        assert_eq!(stub.positions(), &[10, -5, 0]);
        assert_eq!(stub.pulses.len(), 3);
    }
}
