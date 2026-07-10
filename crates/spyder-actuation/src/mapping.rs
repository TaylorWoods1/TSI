//! Map cable length deltas to motor commands.

use crate::motor::Motor;
use crate::winch::Winch;

/// Per-cable actuation command.
#[derive(Clone, Debug)]
pub struct MotorCommand {
    /// Winch rotation in radians.
    pub winch_radians: f64,
    /// Motor steps (rounded to nearest integer).
    pub steps: i64,
    /// Fractional steps before rounding (for diagnostics).
    pub steps_exact: f64,
}

/// Map a length delta through winch + motor.
pub fn length_delta_to_command(
    winch: &Winch,
    motor: &Motor,
    delta_length: f64,
) -> MotorCommand {
    let winch_radians = winch.length_delta_to_radians(delta_length);
    let steps_exact = motor.winch_radians_to_steps(winch_radians);
    MotorCommand {
        winch_radians,
        steps: steps_exact.round() as i64,
        steps_exact,
    }
}

/// Synchronized timing: given step counts, return per-motor delays so the
/// slowest motor finishes in `duration_secs` with constant step rate.
///
/// Returns seconds between steps for each motor (0 if steps == 0).
pub fn synchronized_step_delays(steps: &[i64], duration_secs: f64) -> Vec<f64> {
    let max_abs = steps.iter().map(|s| s.abs()).max().unwrap_or(0);
    if max_abs == 0 || duration_secs <= 0.0 {
        return vec![0.0; steps.len()];
    }
    steps
        .iter()
        .map(|s| {
            if *s == 0 {
                0.0
            } else {
                duration_secs / (s.abs() as f64)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn length_to_steps() {
        // radius 0.05 m, 200 steps/rev, ΔL = 2π*0.05 => one winch rev => 200 steps
        let winch = Winch::new(0.05, 1.0).unwrap();
        let motor = Motor::new(200.0, 1.0).unwrap();
        let delta = 2.0 * std::f64::consts::PI * 0.05;
        let cmd = length_delta_to_command(&winch, &motor, delta);
        assert_eq!(cmd.steps, 200);
        assert_relative_eq!(cmd.winch_radians, 2.0 * std::f64::consts::PI, epsilon = 1e-9);
    }
}
