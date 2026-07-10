//! Forward kinematics: measured lengths → pose.

use crate::anchor::{Anchor, PlatformAttachment};
use crate::error::{Result, SpyderError};
use crate::pose::Pose;
use crate::types::{UnitQuat, Vec3};

/// Which FK algorithm produced a solution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FkMethod {
    /// Numerical Gauss-Newton on point-mass position.
    NumericPointMass,
    /// Analytic trilateration (3 cables).
    Analytic3,
    /// Analytic / reduced path for rectangular 4-cable point-mass.
    AnalyticRect4,
    /// Numerical Gauss-Newton on platform pose (6 DOF).
    NumericPlatform6,
}

/// Result of a forward kinematics solve.
#[derive(Clone, Debug)]
pub struct FkResult {
    /// Recovered position (point-mass) or platform origin.
    pub position: Vec3,
    /// Recovered orientation (identity for point-mass solvers).
    pub orientation: UnitQuat,
    /// Residual norm of length errors.
    pub residual: f64,
    /// Iterations used (0 for pure analytic).
    pub iterations: usize,
    /// Algorithm used.
    pub method: FkMethod,
}

impl FkResult {
    /// Pose view of the solution.
    pub fn pose(&self) -> Pose {
        Pose::new(self.position, self.orientation)
    }
}

fn lengths_at_pose(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
) -> Result<Vec<f64>> {
    let mut out = Vec::with_capacity(anchors.len());
    for (anchor, att) in anchors.iter().zip(attachments.iter()) {
        let b = pose.transform_point(&att.body_point);
        let dist = (b - anchor.exit).norm();
        if dist <= f64::EPSILON {
            return Err(SpyderError::Geometry(
                "FK iterate coincides with an anchor".into(),
            ));
        }
        out.push(dist);
    }
    Ok(out)
}

/// Numerical 6-DOF platform FK via Gauss–Newton with multiplicative orientation updates.
///
/// Minimizes \(\sum_i (\|p + R b_i - a_i\| - L_i)^2\). Needs \(m \ge 3\); full
/// orientation observability typically needs \(m \ge 6\) or rich attachments.
pub fn fk_platform_numeric(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    lengths: &[f64],
    seed: &Pose,
) -> Result<FkResult> {
    if anchors.len() != lengths.len()
        || anchors.len() != attachments.len()
        || anchors.len() < 3
    {
        return Err(SpyderError::Config(
            "platform FK needs matching anchors/attachments/lengths with n >= 3".into(),
        ));
    }

    let mut pose = seed.clone();
    let max_iters = 100;
    let eps = 1e-7;
    let mut residual = f64::INFINITY;
    let mut iterations = 0;
    let m = lengths.len();
    let mut lambda = 1e-3; // Levenberg–Marquardt damping

    for iter in 0..max_iters {
        iterations = iter + 1;
        let pred = lengths_at_pose(anchors, attachments, &pose)?;
        let mut r = nalgebra::DVector::zeros(m);
        residual = 0.0;
        for i in 0..m {
            let e = pred[i] - lengths[i];
            r[i] = e;
            residual += e * e;
        }
        residual = residual.sqrt();
        if residual < 1e-10 {
            return Ok(FkResult {
                position: pose.position,
                orientation: pose.orientation,
                residual,
                iterations,
                method: FkMethod::NumericPlatform6,
            });
        }

        // Jacobian m×6: columns 0..2 = ∂L/∂p, 3..5 = ∂L/∂ω (right multiplicative)
        let mut j = nalgebra::DMatrix::zeros(m, 6);
        for k in 0..3 {
            let mut pose_p = pose.clone();
            pose_p.position[k] += eps;
            let pred_p = lengths_at_pose(anchors, attachments, &pose_p)?;
            for i in 0..m {
                j[(i, k)] = (pred_p[i] - pred[i]) / eps;
            }
        }
        for k in 0..3 {
            let mut dw = Vec3::zeros();
            dw[k] = eps;
            let pose_p = Pose::new(
                pose.position,
                pose.orientation * UnitQuat::from_scaled_axis(dw),
            );
            let pred_p = lengths_at_pose(anchors, attachments, &pose_p)?;
            for i in 0..m {
                j[(i, 3 + k)] = (pred_p[i] - pred[i]) / eps;
            }
        }

        let jtj = j.transpose() * &j;
        let mut a = jtj.clone();
        for i in 0..6 {
            a[(i, i)] += lambda;
        }
        let jtr = j.transpose() * &r;
        let delta = match nalgebra::linalg::SVD::new(a, true, true).solve(&jtr, 1e-12) {
            Ok(d) => d,
            Err(_) => {
                lambda *= 10.0;
                continue;
            }
        };

        let candidate = Pose::new(
            pose.position - Vec3::new(delta[0], delta[1], delta[2]),
            pose.orientation
                * UnitQuat::from_scaled_axis(-Vec3::new(delta[3], delta[4], delta[5])),
        );
        let pred_c = lengths_at_pose(anchors, attachments, &candidate)?;
        let mut res_c = 0.0;
        for i in 0..m {
            let e = pred_c[i] - lengths[i];
            res_c += e * e;
        }
        res_c = res_c.sqrt();

        if res_c < residual {
            pose = candidate;
            lambda = (lambda * 0.3).max(1e-9);
            if delta.norm() < 1e-12 {
                return Ok(FkResult {
                    position: pose.position,
                    orientation: pose.orientation,
                    residual: res_c,
                    iterations,
                    method: FkMethod::NumericPlatform6,
                });
            }
        } else {
            lambda *= 4.0;
            if lambda > 1e8 {
                break;
            }
        }
    }

    Err(SpyderError::FkNonConvergence {
        residual,
        iterations,
    })
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
    let mut lambda = 1e-3; // Levenberg–Marquardt damping for singular configs

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

        let mut a_mat = jtj;
        for i in 0..3 {
            a_mat[(i, i)] += lambda;
        }
        let delta = match nalgebra::linalg::SVD::new(a_mat, true, true).solve(&jtr, 1e-12) {
            Ok(d) => d,
            Err(_) => {
                lambda *= 10.0;
                if lambda > 1e8 {
                    break;
                }
                continue;
            }
        };

        let candidate = p - delta;
        let mut res_c = 0.0;
        for (a, &l_meas) in anchors.iter().zip(lengths.iter()) {
            let err = (candidate - a).norm() - l_meas;
            res_c += err * err;
        }
        res_c = res_c.sqrt();

        if res_c < residual {
            p = candidate;
            lambda = (lambda * 0.3).max(1e-9);
            if delta.norm() < tol || res_c < 1e-10 {
                return Ok(FkResult {
                    position: p,
                    orientation: UnitQuat::identity(),
                    residual: res_c,
                    iterations,
                    method: FkMethod::NumericPointMass,
                });
            }
        } else {
            lambda *= 4.0;
            if lambda > 1e8 {
                break;
            }
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
