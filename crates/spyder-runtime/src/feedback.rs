//! Closed-loop feedback helpers: steps → lengths → FK pose.

use spyder_core::{Robot, Vec3};

use crate::{Axis, Result, RuntimeError};

/// Convert cumulative motor steps to cable lengths using home reference.
pub fn lengths_from_steps(
    home_lengths: &[f64],
    steps: &[i64],
    axes: &[Axis],
) -> Result<Vec<f64>> {
    if home_lengths.len() != steps.len() || steps.len() != axes.len() {
        return Err(RuntimeError::Config(
            "home/steps/axes length mismatch".into(),
        ));
    }
    let mut out = Vec::with_capacity(steps.len());
    for i in 0..steps.len() {
        // Inverse of Motor::winch_radians_to_steps:
        // steps = (winch_rad / 2π) * gear_ratio * steps_per_rev
        // winch_rad = steps / steps_per_rev / gear_ratio * 2π
        let winch_rad = (steps[i] as f64 / axes[i].motor.steps_per_rev)
            / axes[i].motor.gear_ratio
            * (2.0 * std::f64::consts::PI);
        let delta_l = axes[i].winch.radians_to_length_delta(winch_rad);
        out.push(home_lengths[i] + delta_l);
    }
    Ok(out)
}

/// Estimate Cartesian pose from measured steps via FK.
pub fn pose_from_steps(
    robot: &Robot,
    home_lengths: &[f64],
    steps: &[i64],
    axes: &[Axis],
    seed: Vec3,
) -> Result<Vec3> {
    let lengths = lengths_from_steps(home_lengths, steps, axes)?;
    let fk = robot.fk(&lengths, seed)?;
    Ok(fk.position)
}

/// Corrective length error: commanded − measured.
pub fn length_error(commanded: &[f64], measured: &[f64]) -> Result<Vec<f64>> {
    if commanded.len() != measured.len() {
        return Err(RuntimeError::Config("length vector mismatch".into()));
    }
    Ok(commanded
        .iter()
        .zip(measured.iter())
        .map(|(c, m)| c - m)
        .collect())
}

/// Build identical axes for `n` cables.
pub fn uniform_axes(n: usize, drum: f64, steps_per_rev: f64) -> Result<Vec<Axis>> {
    (0..n)
        .map(|_| Axis::new(drum, steps_per_rev, 1.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use spyder_core::{Pose, Preset, Robot};

    #[test]
    fn steps_roundtrip_to_pose() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let home = Vec3::new(0.0, 0.0, 1.5);
        let home_ik = robot.ik(&Pose::from_position(home)).unwrap();
        let axes = uniform_axes(4, 0.05, 200.0).unwrap();
        let p = pose_from_steps(&robot, &home_ik.lengths, &[0, 0, 0, 0], &axes, home).unwrap();
        assert_relative_eq!(p.x, home.x, epsilon = 1e-5);
        assert_relative_eq!(p.y, home.y, epsilon = 1e-5);
        assert_relative_eq!(p.z, home.z, epsilon = 1e-5);
    }
}
