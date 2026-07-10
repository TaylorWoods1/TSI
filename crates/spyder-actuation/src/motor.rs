//! Stepper / encoder motor parameters.

use thiserror::Error;

/// Motor configuration errors.
#[derive(Debug, Error)]
pub enum MotorError {
    /// Invalid parameters.
    #[error("{0}")]
    Config(String),
}

/// Rotary actuator behind a winch.
#[derive(Clone, Debug)]
pub struct Motor {
    /// Steps (or encoder counts) per motor revolution.
    pub steps_per_rev: f64,
    /// Gear ratio: motor_turns / winch_turns (1.0 = direct drive).
    pub gear_ratio: f64,
}

impl Motor {
    /// Construct with positive steps and gear ratio.
    pub fn new(steps_per_rev: f64, gear_ratio: f64) -> Result<Self, MotorError> {
        if steps_per_rev <= 0.0 {
            return Err(MotorError::Config("steps_per_rev must be > 0".into()));
        }
        if gear_ratio <= 0.0 {
            return Err(MotorError::Config("gear_ratio must be > 0".into()));
        }
        Ok(Self {
            steps_per_rev,
            gear_ratio,
        })
    }

    /// Convert winch radians to motor steps (can be fractional before rounding).
    pub fn winch_radians_to_steps(&self, winch_radians: f64) -> f64 {
        let motor_revs = (winch_radians / (2.0 * std::f64::consts::PI)) * self.gear_ratio;
        motor_revs * self.steps_per_rev
    }
}
