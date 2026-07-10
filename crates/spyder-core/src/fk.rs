//! Forward kinematics: measured lengths → pose.

use crate::anchor::Anchor;
use crate::error::{Result, SpyderError};
use crate::types::Vec3;

/// Which FK algorithm produced a solution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FkMethod {
    /// Numerical Gauss-Newton on point-mass position.
    NumericPointMass,
    /// Analytic trilateration (3 cables).
    Analytic3,
    /// Analytic / reduced path for rectangular 4-cable point-mass.
    AnalyticRect4,
}

/// Result of a forward kinematics solve.
#[derive(Clone, Debug)]
pub struct FkResult {
    /// Recovered position (point-mass) or platform origin.
    pub position: Vec3,
    /// Residual norm of length errors.
    pub residual: f64,
    /// Iterations used (0 for pure analytic).
    pub iterations: usize,
    /// Algorithm used.
    pub method: FkMethod,
}

/// Numerical point-mass FK via Gauss-Newton.
///
/// Minimizes \(\sum_i (\|p - a_i\| - L_i)^2\).
pub fn fk_point_mass_numeric(
    anchors: &[Vec3],
    lengths: &[f64],
    seed: Vec3,
) -> Result<FkResult> {
    if anchors.len() != lengths.len() || anchors.len() < 3 {
        return Err(SpyderError::Config(
            "FK needs matching anchors/lengths with n >= 3".into(),
        ));
    }

    let mut p = seed;
    let max_iters = 50;
    let tol = 1e-12;
    let mut residual = f64::INFINITY;
    let mut iterations = 0;

    for iter in 0..max_iters {
        iterations = iter + 1;
        let mut jtj = nalgebra::Matrix3::zeros();
        let mut jtr = Vec3::zeros();
        residual = 0.0;

        for (a, &l_meas) in anchors.iter().zip(lengths.iter()) {
            let diff = p - a;
            let dist = diff.norm();
            if dist <= f64::EPSILON {
                return Err(SpyderError::Geometry(
                    "FK iterate coincides with an anchor".into(),
                ));
            }
            let err = dist - l_meas;
            residual += err * err;
            let u = diff / dist; // ∂||p-a||/∂p
            jtj += u * u.transpose();
            jtr += u * err;
        }
        residual = residual.sqrt();

        // Solve JᵀJ Δ = Jᵀr  (Gauss-Newton step: p ← p - Δ)
        let delta = nalgebra::linalg::SVD::new(jtj, true, true)
            .solve(&jtr, 1e-12)
            .map_err(|_| SpyderError::SingularStructure)?;
        p -= delta;

        if delta.norm() < tol || residual < 1e-10 {
            return Ok(FkResult {
                position: p,
                residual,
                iterations,
                method: FkMethod::NumericPointMass,
            });
        }
    }

    Err(SpyderError::FkNonConvergence {
        residual,
        iterations,
    })
}

/// Point-mass FK from [`Anchor`] exits.
pub fn fk_point_mass_from_anchors(
    anchors: &[Anchor],
    lengths: &[f64],
    seed: Vec3,
) -> Result<FkResult> {
    let exits: Vec<Vec3> = anchors.iter().map(|a| a.exit).collect();
    fk_point_mass_numeric(&exits, lengths, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ik::ideal_ik_point_mass;
    use crate::preset::rect;
    use approx::assert_relative_eq;

    #[test]
    fn ik_fk_round_trip_point_mass() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let exits: Vec<Vec3> = anchors.iter().map(|a| a.exit).collect();
        let p = Vec3::new(0.3, -0.2, 1.0);
        let lengths = ideal_ik_point_mass(&exits, &p).unwrap();
        let recovered =
            fk_point_mass_numeric(&exits, &lengths, Vec3::new(0.0, 0.0, 1.5)).unwrap();
        assert_relative_eq!(recovered.position.x, p.x, epsilon = 1e-6);
        assert_relative_eq!(recovered.position.y, p.y, epsilon = 1e-6);
        assert_relative_eq!(recovered.position.z, p.z, epsilon = 1e-6);
        assert_eq!(recovered.method, FkMethod::NumericPointMass);
    }
}
