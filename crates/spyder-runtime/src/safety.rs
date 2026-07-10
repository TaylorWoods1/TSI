//! Soft limits, e-stop, slack / overspeed guards.

use spyder_core::Vec3;
use thiserror::Error;

use crate::{Result, RuntimeError};

/// Safety policy applied before/during motion.
#[derive(Clone, Debug)]
pub struct SafetyLimits {
    /// Axis-aligned workspace box (inclusive).
    pub min: Vec3,
    /// Axis-aligned workspace box (inclusive).
    pub max: Vec3,
    /// Max Cartesian speed (m/s) for planned segments.
    pub max_speed_mps: f64,
    /// Max |steps| commanded in a single backend move.
    pub max_steps_per_move: i64,
    /// Minimum allowed cable length (m) — below this is treated as over-retract.
    pub min_cable_length_m: f64,
    /// Maximum allowed cable length (m).
    pub max_cable_length_m: f64,
    /// If true, motion is blocked until cleared.
    pub estop: bool,
}

impl Default for SafetyLimits {
    fn default() -> Self {
        Self {
            min: Vec3::new(-5.0, -5.0, 0.1),
            max: Vec3::new(5.0, 5.0, 10.0),
            max_speed_mps: 1.0,
            max_steps_per_move: 50_000,
            min_cable_length_m: 0.15,
            max_cable_length_m: 30.0,
            estop: false,
        }
    }
}

/// Safety violations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SafetyError {
    /// Emergency stop latched.
    #[error("e-stop active")]
    Estop,
    /// Pose outside soft box.
    #[error("pose outside soft limits: ({x:.3},{y:.3},{z:.3})")]
    OutOfBounds {
        /// X
        x: f64,
        /// Y
        y: f64,
        /// Z
        z: f64,
    },
    /// Planned segment too fast.
    #[error("segment speed {speed:.3} m/s exceeds max {max:.3}")]
    Overspeed {
        /// Planned speed
        speed: f64,
        /// Limit
        max: f64,
    },
    /// Step burst too large.
    #[error("steps {steps} exceed max_steps_per_move {max}")]
    StepBurst {
        /// Requested
        steps: i64,
        /// Limit
        max: i64,
    },
    /// Cable too short (possible crash / wrap).
    #[error("cable {index} length {length:.3} m below minimum {min:.3}")]
    CableTooShort {
        /// Cable index
        index: usize,
        /// Length
        length: f64,
        /// Min
        min: f64,
    },
    /// Cable too long (possible slack).
    #[error("cable {index} length {length:.3} m above maximum {max:.3} (slack risk)")]
    CableTooLong {
        /// Cable index
        index: usize,
        /// Length
        length: f64,
        /// Max
        max: f64,
    },
}

impl From<SafetyError> for RuntimeError {
    fn from(e: SafetyError) -> Self {
        RuntimeError::Safety(e.to_string())
    }
}

impl SafetyLimits {
    /// Latch e-stop.
    pub fn trip_estop(&mut self) {
        self.estop = true;
    }

    /// Clear e-stop (operator acknowledge).
    pub fn clear_estop(&mut self) {
        self.estop = false;
    }

    /// Ensure e-stop is not latched.
    pub fn check_estop(&self) -> Result<()> {
        if self.estop {
            Err(SafetyError::Estop.into())
        } else {
            Ok(())
        }
    }

    /// Pose inside soft box.
    pub fn check_pose(&self, p: &Vec3) -> Result<()> {
        self.check_estop()?;
        if p.x < self.min.x
            || p.y < self.min.y
            || p.z < self.min.z
            || p.x > self.max.x
            || p.y > self.max.y
            || p.z > self.max.z
        {
            return Err(SafetyError::OutOfBounds {
                x: p.x,
                y: p.y,
                z: p.z,
            }
            .into());
        }
        Ok(())
    }

    /// Segment speed = distance / duration.
    pub fn check_segment(&self, a: &Vec3, b: &Vec3, duration_s: f64) -> Result<()> {
        self.check_estop()?;
        if duration_s <= 0.0 {
            return Err(RuntimeError::Config("duration must be > 0".into()));
        }
        let speed = (b - a).norm() / duration_s;
        if speed > self.max_speed_mps + 1e-9 {
            return Err(SafetyError::Overspeed {
                speed,
                max: self.max_speed_mps,
            }
            .into());
        }
        Ok(())
    }

    /// Cable length envelope (slack / over-retract).
    pub fn check_lengths(&self, lengths: &[f64]) -> Result<()> {
        self.check_estop()?;
        for (i, &l) in lengths.iter().enumerate() {
            if l < self.min_cable_length_m {
                return Err(SafetyError::CableTooShort {
                    index: i,
                    length: l,
                    min: self.min_cable_length_m,
                }
                .into());
            }
            if l > self.max_cable_length_m {
                return Err(SafetyError::CableTooLong {
                    index: i,
                    length: l,
                    max: self.max_cable_length_m,
                }
                .into());
            }
        }
        Ok(())
    }

    /// Per-move step magnitude guard.
    pub fn check_steps(&self, steps: &[i64]) -> Result<()> {
        self.check_estop()?;
        for &s in steps {
            let a = s.abs();
            if a > self.max_steps_per_move {
                return Err(SafetyError::StepBurst {
                    steps: a,
                    max: self.max_steps_per_move,
                }
                .into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_bounds_and_estop() {
        let mut s = SafetyLimits::default();
        assert!(s.check_pose(&Vec3::new(0.0, 0.0, 1.0)).is_ok());
        assert!(s.check_pose(&Vec3::new(100.0, 0.0, 1.0)).is_err());
        s.trip_estop();
        assert!(s.check_pose(&Vec3::new(0.0, 0.0, 1.0)).is_err());
        s.clear_estop();
        assert!(s.check_pose(&Vec3::new(0.0, 0.0, 1.0)).is_ok());
    }

    #[test]
    fn rejects_overspeed_and_slack() {
        let s = SafetyLimits {
            max_speed_mps: 0.5,
            ..SafetyLimits::default()
        };
        let a = Vec3::new(0.0, 0.0, 1.0);
        let b = Vec3::new(2.0, 0.0, 1.0);
        assert!(s.check_segment(&a, &b, 1.0).is_err()); // 2 m/s
        assert!(s.check_segment(&a, &b, 5.0).is_ok());
        assert!(s.check_lengths(&[0.05, 1.0, 1.0, 1.0]).is_err());
        assert!(s.check_lengths(&[40.0, 1.0, 1.0, 1.0]).is_err());
    }

    #[test]
    fn rejects_step_burst_over_limit() {
        let s = SafetyLimits {
            max_steps_per_move: 100,
            ..SafetyLimits::default()
        };
        assert!(s.check_steps(&[50, 50, 0, 0]).is_ok());
        assert!(matches!(
            s.check_steps(&[200, 0, 0, 0]).unwrap_err(),
            RuntimeError::Safety(msg) if msg.contains("steps")
        ));
    }
}
