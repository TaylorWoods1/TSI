//! Forward kinematics: measured lengths → pose.

use nalgebra::{DMatrix, DVector};

use crate::anchor::{Anchor, PlatformAttachment};
use crate::cable_eval::{default_pulley_radius, predicted_lengths};
use crate::error::{Result, SpyderError};
use crate::jacobian::length_jacobian_platform_6_with_pulls;
use crate::pose::Pose;
use crate::robot::CableModelKind;
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

/// Options for forward kinematics.
#[derive(Clone, Debug, Default)]
pub struct FkOptions {
    /// Allow platform FK with fewer than 6 cables (orientation may be unobservable).
    pub allow_underconstrained: bool,
    /// Per-cable tensions for sag model FK (optional).
    pub tensions: Option<Vec<f64>>,
}

impl FkOptions {
    /// Strict defaults: platform FK requires m >= 6.
    pub fn strict() -> Self {
        Self::default()
    }

    /// Permit underconstrained platform FK (seed-dependent orientation).
    pub fn permissive() -> Self {
        Self {
            allow_underconstrained: true,
            ..Self::default()
        }
    }
}

fn residual_vec(pred: &[f64], meas: &[f64]) -> (DVector<f64>, f64) {
    let m = meas.len();
    let mut r = DVector::zeros(m);
    let mut sum = 0.0;
    for i in 0..m {
        let e = pred[i] - meas[i];
        r[i] = e;
        sum += e * e;
    }
    (r, sum.sqrt())
}

/// Numerical 6-DOF platform FK with model-aware length prediction.
pub fn fk_platform_numeric(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    lengths: &[f64],
    seed: &Pose,
    cable_model: &CableModelKind,
    opts: &FkOptions,
) -> Result<FkResult> {
    if anchors.len() != lengths.len()
        || anchors.len() != attachments.len()
        || anchors.len() < 3
    {
        return Err(SpyderError::Config(
            "platform FK needs matching anchors/attachments/lengths with m >= 3".into(),
        ));
    }
    if !opts.allow_underconstrained && anchors.len() < 6 {
        return Err(SpyderError::Config(
            "platform FK requires m >= 6 cables for full 6-DOF observability \
             (set FkOptions.allow_underconstrained = true to override)"
                .into(),
        ));
    }

    let def_r = default_pulley_radius(cable_model);
    let tensions = opts.tensions.as_deref();
        let use_analytic_j = false; // FD is more robust across attachment offsets

    let mut pose = seed.clone();
    let max_iters = 100;
    let eps = 1e-7;
    let mut residual = f64::INFINITY;
    let mut iterations = 0;
    let m = lengths.len();
    let mut lambda = 1e-3;

    for iter in 0..max_iters {
        iterations = iter + 1;
        let pred = predicted_lengths(anchors, attachments, &pose, cable_model, tensions, def_r)?;
        let (r, res) = residual_vec(&pred, lengths);
        residual = res;
        if residual < 1e-10 {
            return Ok(FkResult {
                position: pose.position,
                orientation: pose.orientation,
                residual,
                iterations,
                method: FkMethod::NumericPlatform6,
            });
        }

        let j = if use_analytic_j {
            let pulls = crate::cable_eval::unit_pulls_at_pose(
                anchors,
                attachments,
                &pose,
                cable_model,
                tensions,
                def_r,
            )?;
            length_jacobian_platform_6_with_pulls(anchors, attachments, &pose, &pulls)?
        } else {
            let mut j = DMatrix::zeros(m, 6);
            for k in 0..3 {
                let mut pose_p = pose.clone();
                pose_p.position[k] += eps;
                let pred_p =
                    predicted_lengths(anchors, attachments, &pose_p, cable_model, tensions, def_r)?;
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
                let pred_p =
                    predicted_lengths(anchors, attachments, &pose_p, cable_model, tensions, def_r)?;
                for i in 0..m {
                    j[(i, 3 + k)] = (pred_p[i] - pred[i]) / eps;
                }
            }
            j
        };

        let jtj = j.transpose() * &j;
        let mut a_mat = jtj.clone();
        for i in 0..6 {
            a_mat[(i, i)] += lambda;
        }
        let jtr = j.transpose() * &r;
        let delta = match nalgebra::linalg::SVD::new(a_mat, true, true).solve(&jtr, 1e-12) {
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
        let pred_c =
            predicted_lengths(anchors, attachments, &candidate, cable_model, tensions, def_r)?;
        let (_, res_c) = residual_vec(&pred_c, lengths);

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

/// Numerical point-mass FK with model-aware length prediction.
pub fn fk_point_mass_numeric_model(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    lengths: &[f64],
    seed: Vec3,
    cable_model: &CableModelKind,
    opts: &FkOptions,
) -> Result<FkResult> {
    if anchors.len() != lengths.len() || anchors.len() < 3 {
        return Err(SpyderError::Config(
            "FK needs matching anchors/lengths with n >= 3".into(),
        ));
    }
    let atts: Vec<_> = if attachments.is_empty() {
        (0..anchors.len())
            .map(|_| PlatformAttachment::origin())
            .collect()
    } else {
        attachments.to_vec()
    };
    if atts.len() != anchors.len() {
        return Err(SpyderError::Config(
            "attachments must match anchor count".into(),
        ));
    }

    let def_r = default_pulley_radius(cable_model);
    let tensions = opts.tensions.as_deref();
    let mut p = seed;
    let max_iters = 50;
    let tol = 1e-12;
    let mut residual = f64::INFINITY;
    let mut iterations = 0;
    let mut lambda = 1e-3;
    let use_fd = !matches!(cable_model, CableModelKind::Ideal);
    let eps = 1e-7;

    for _iter in 0..max_iters {
        iterations += 1;
        let pose = Pose::from_position(p);
        let pred = predicted_lengths(anchors, &atts, &pose, cable_model, tensions, def_r)?;
        let (_, res) = residual_vec(&pred, lengths);
        residual = res;

        let delta = if use_fd {
            let mut j = DMatrix::zeros(anchors.len(), 3);
            for k in 0..3 {
                let mut dp = Vec3::zeros();
                dp[k] = eps;
                let pred_p = predicted_lengths(
                    anchors,
                    &atts,
                    &Pose::from_position(p + dp),
                    cable_model,
                    tensions,
                    def_r,
                )?;
                for i in 0..anchors.len() {
                    j[(i, k)] = (pred_p[i] - pred[i]) / eps;
                }
            }
            let r = DVector::from_iterator(
                anchors.len(),
                pred.iter().zip(lengths.iter()).map(|(a, b)| a - b),
            );
            let jtj = j.transpose() * &j;
            let mut a_mat = jtj.clone();
            for i in 0..3 {
                a_mat[(i, i)] += lambda;
            }
            let jtr = j.transpose() * &r;
            let d = nalgebra::linalg::SVD::new(a_mat, true, true)
                .solve(&jtr, 1e-12)
                .map_err(|_| SpyderError::SingularStructure)?;
            Vec3::new(d[0], d[1], d[2])
        } else {
            let mut jtj = nalgebra::Matrix3::zeros();
            let mut jtr = Vec3::zeros();
            for (i, anchor) in anchors.iter().enumerate() {
                let diff = p - anchor.exit;
                let dist = diff.norm();
                if dist <= f64::EPSILON {
                    return Err(SpyderError::Geometry(
                        "FK iterate coincides with an anchor".into(),
                    ));
                }
                let u = diff / dist;
                let err = pred[i] - lengths[i];
                jtj += u * u.transpose();
                jtr += u * err;
            }
            let mut a_mat = jtj;
            for i in 0..3 {
                a_mat[(i, i)] += lambda;
            }
            nalgebra::linalg::SVD::new(a_mat, true, true)
                .solve(&jtr, 1e-12)
                .map_err(|_| SpyderError::SingularStructure)?
        };

        let candidate = p - delta;
        let pred_c = predicted_lengths(
            anchors,
            &atts,
            &Pose::from_position(candidate),
            cable_model,
            tensions,
            def_r,
        )?;
        let (_, res_c) = residual_vec(&pred_c, lengths);

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

/// Numerical point-mass FK via Gauss–Newton (ideal chord, legacy API).
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
    let anchor_objs: Vec<Anchor> = anchors.iter().map(|&e| Anchor::point(e)).collect();
    fk_point_mass_numeric_model(
        &anchor_objs,
        &[],
        lengths,
        seed,
        &CableModelKind::Ideal,
        &FkOptions::default(),
    )
}

/// Point-mass FK from [`Anchor`] exits.
pub fn fk_point_mass_from_anchors(
    anchors: &[Anchor],
    lengths: &[f64],
    seed: Vec3,
    cable_model: &CableModelKind,
    opts: &FkOptions,
) -> Result<FkResult> {
    fk_point_mass_numeric_model(anchors, &[], lengths, seed, cable_model, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::PlatformAttachment;
    use crate::cable_eval::predicted_lengths;
    use crate::ik::ideal_ik_point_mass;
    use crate::preset::rect;
    use crate::robot::CableModelKind;
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

    #[test]
    fn pulley_ik_fk_round_trip() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let mut pulley_anchors = anchors.clone();
        for a in &mut pulley_anchors {
            a.pulley_axis = Some(Vec3::z());
            a.pulley_radius = 0.06;
        }
        let pose = Pose::from_position(Vec3::new(0.2, -0.1, 1.0));
        let model = CableModelKind::Pulley {
            default_radius: 0.06,
        };
        let ik_lens = predicted_lengths(
            &pulley_anchors,
            &vec![PlatformAttachment::origin(); 4],
            &pose,
            &model,
            None,
            0.06,
        )
        .unwrap();
        let fk = fk_point_mass_from_anchors(
            &pulley_anchors,
            &ik_lens,
            Vec3::new(0.0, 0.0, 1.5),
            &model,
            &FkOptions::default(),
        )
        .unwrap();
        assert_relative_eq!(fk.position.x, pose.position.x, epsilon = 1e-4);
        assert_relative_eq!(fk.position.y, pose.position.y, epsilon = 1e-4);
        assert_relative_eq!(fk.position.z, pose.position.z, epsilon = 1e-4);
    }
}
