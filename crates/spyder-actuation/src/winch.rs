//! Winch geometry.

use thiserror::Error;

/// Errors from winch mapping.
#[derive(Debug, Error)]
pub enum WinchError {
    /// Invalid parameters.
    #[error("{0}")]
    Config(String),
}

/// Constant-radius drum winch.
#[derive(Clone, Debug)]
pub struct Winch {
    /// Drum radius in meters.
    pub radius: f64,
    /// +1 or -1: sign relating positive length pay-out to positive rotation.
    pub direction: f64,
}

impl Winch {
    /// Create a winch with positive radius.
    pub fn new(radius: f64, direction: f64) -> Result<Self, WinchError> {
        if radius <= 0.0 {
            return Err(WinchError::Config("radius must be > 0".into()));
        }
        if direction.abs() < f64::EPSILON {
            return Err(WinchError::Config("direction must be non-zero".into()));
        }
        Ok(Self {
            radius,
            direction: direction.signum(),
        })
    }

    /// Convert cable length change (meters, positive = longer / pay out) to drum radians.
    pub fn length_delta_to_radians(&self, delta_length: f64) -> f64 {
        self.direction * delta_length / self.radius
    }

    /// Convert drum radians to cable length change.
    pub fn radians_to_length_delta(&self, radians: f64) -> f64 {
        self.direction * radians * self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_non_positive_radius() {
        assert!(Winch::new(0.0, 1.0).is_err());
    }

    #[test]
    fn rejects_zero_direction() {
        assert!(Winch::new(0.05, 0.0).is_err());
    }

    #[test]
    fn length_radians_round_trip() {
        let w = Winch::new(0.05, 1.0).unwrap();
        let delta = 0.25;
        let rad = w.length_delta_to_radians(delta);
        assert_relative_eq!(w.radians_to_length_delta(rad), delta);
    }

    #[test]
    fn negative_direction_flips_sign() {
        let w = Winch::new(0.05, -1.0).unwrap();
        assert!(w.length_delta_to_radians(0.1) < 0.0);
    }
}
