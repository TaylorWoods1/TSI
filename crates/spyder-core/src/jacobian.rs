//! Length Jacobian for cable robots.

use nalgebra::DMatrix;

use crate::anchor::{Anchor, PlatformAttachment};
use crate::error::{Result, SpyderError};
use crate::pose::Pose;
use crate::types::Vec3;

/// Point-mass length Jacobian \(J \in \mathbb{R}^{m \times 3}\) such that
/// \(\dot L \approx J\, v\) for platform velocity \(v\).
///
/// Row \(i\) is \(-u_i^\top\) where \(u_i = (a_i - p)/\|a_i - p\|\) (unit vector
/// from the end-effector toward anchor \(i\)). Moving toward an anchor shortens
/// that cable (\(\dot L_i < 0\)).
pub fn length_jacobian_point_mass(anchors: &[Vec3], position: &Vec3) -> Result<DMatrix<f64>> {
    let m = anchors.len();
    if m < 3 {
        return Err(SpyderError::Config("need at least 3 anchors".into()));
    }
    let mut j = DMatrix::zeros(m, 3);
    for (i, a) in anchors.iter().enumerate() {
        let diff = a - position;
        let dist = diff.norm();
        if dist <= f64::EPSILON {
            return Err(SpyderError::Geometry(format!(
                "zero-length cable at index {i}"
            )));
        }
        let u = diff / dist;
        j[(i, 0)] = -u.x;
        j[(i, 1)] = -u.y;
        j[(i, 2)] = -u.z;
    }
    Ok(j)
}

/// Length Jacobian from anchors + pose (point-mass or platform attachments).
///
/// For platform mode, uses world attachment points \(B_i = p + R b_i\) and still
/// returns an \(m \times 3\) translational Jacobian (orientation twist omitted).
/// For full 6-DOF twist use [`length_jacobian_platform_6`].
pub fn length_jacobian(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
) -> Result<DMatrix<f64>> {
    if anchors.len() != attachments.len() {
        return Err(SpyderError::Config(
            "anchors/attachments length mismatch".into(),
        ));
    }
    let points: Vec<Vec3> = attachments
        .iter()
        .map(|att| pose.transform_point(&att.body_point))
        .collect();
    // Reuse point-mass helper but with per-cable attachment positions as "p".
    // Build rows manually because each cable has its own B_i.
    let m = anchors.len();
    let mut j = DMatrix::zeros(m, 3);
    for (i, (anchor, b)) in anchors.iter().zip(points.iter()).enumerate() {
        let diff = anchor.exit - b;
        let dist = diff.norm();
        if dist <= f64::EPSILON {
            return Err(SpyderError::Geometry(format!(
                "zero-length cable at index {i}"
            )));
        }
        let u = diff / dist;
        j[(i, 0)] = -u.x;
        j[(i, 1)] = -u.y;
        j[(i, 2)] = -u.z;
    }
    Ok(j)
}

/// Full platform length Jacobian \(J \in \mathbb{R}^{m \times 6}\) such that
/// \(\dot L \approx J\, \xi\) for platform twist \(\xi = [\dot{\mathbf{p}};\, \boldsymbol{\omega}]\).
///
/// Translational columns 0–2: \(-\mathbf{u}_i^\top\) where \(\mathbf{u}_i = (A_i - B_i)/\|A_i - B_i\|\).
/// Rotational columns 3–5: \(-(\mathbf{b}_i \times R^\top \mathbf{u}_i)^\top\) for
/// body-frame angular velocity (right-multiplicative orientation perturbation).
pub fn length_jacobian_platform_6(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
) -> Result<DMatrix<f64>> {
    if anchors.len() != attachments.len() {
        return Err(SpyderError::Config(
            "anchors/attachments length mismatch".into(),
        ));
    }
    let m = anchors.len();
    if m < 3 {
        return Err(SpyderError::Config("need at least 3 cables".into()));
    }
    let mut j = DMatrix::zeros(m, 6);
    for (i, (anchor, att)) in anchors.iter().zip(attachments.iter()).enumerate() {
        let b = pose.transform_point(&att.body_point);
        let diff = anchor.exit - b;
        let dist = diff.norm();
        if dist <= f64::EPSILON {
            return Err(SpyderError::Geometry(format!(
                "zero-length cable at index {i}"
            )));
        }
        let u = diff / dist;
        // Body-frame angular velocity (right-multiplicative quaternion perturbation):
        // ∂L/∂ω_body = (b × R^T u)^T
        let u_body = pose.orientation.inverse() * u;
        let rot = att.body_point.cross(&u_body);
        j[(i, 0)] = -u.x;
        j[(i, 1)] = -u.y;
        j[(i, 2)] = -u.z;
        j[(i, 3)] = -rot.x;
        j[(i, 4)] = -rot.y;
        j[(i, 5)] = -rot.z;
    }
    Ok(j)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::PlatformAttachment;
    use crate::ik::ideal_ik_point_mass;
    use crate::preset::rect;
    use approx::assert_relative_eq;

    #[test]
    fn jacobian_finite_diff_matches() {
        let exits = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
        ];
        let p = Vec3::new(0.1, -0.05, 0.2);
        let j = length_jacobian_point_mass(&exits, &p).unwrap();
        let l0 = ideal_ik_point_mass(&exits, &p).unwrap();
        let eps = 1e-7;
        for axis in 0..3 {
            let mut dp = Vec3::zeros();
            dp[axis] = eps;
            let l1 = ideal_ik_point_mass(&exits, &(p + dp)).unwrap();
            for i in 0..4 {
                let num = (l1[i] - l0[i]) / eps;
                assert_relative_eq!(j[(i, axis)], num, epsilon = 1e-5);
            }
        }
    }

    #[test]
    fn jacobian_from_anchors_api() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let atts: Vec<_> = (0..4).map(|_| PlatformAttachment::origin()).collect();
        let pose = Pose::from_position(Vec3::new(0.2, 0.0, 1.0));
        let j = length_jacobian(&anchors, &atts, &pose).unwrap();
        assert_eq!(j.nrows(), 4);
        assert_eq!(j.ncols(), 3);
    }

    #[test]
    fn platform_6_jacobian_finite_diff_matches() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let atts: Vec<_> = [
            Vec3::new(0.2, 0.1, 0.0),
            Vec3::new(-0.2, 0.1, 0.0),
            Vec3::new(-0.2, -0.1, 0.0),
            Vec3::new(0.2, -0.1, 0.0),
        ]
        .into_iter()
        .map(PlatformAttachment::at)
        .collect();
        let pose = Pose::new(
            Vec3::new(0.1, -0.05, 1.2),
            crate::types::UnitQuat::from_scaled_axis(Vec3::new(0.03, -0.02, 0.04)),
        );
        let j = length_jacobian_platform_6(&anchors, &atts, &pose).unwrap();
        let eps = 1e-7;

        let lengths_at = |p: &Pose| -> Vec<f64> {
            atts.iter()
                .zip(anchors.iter())
                .map(|(att, anchor)| {
                    let b = p.transform_point(&att.body_point);
                    (b - anchor.exit).norm()
                })
                .collect()
        };
        let l0 = lengths_at(&pose);

        for axis in 0..3 {
            let mut dp = Vec3::zeros();
            dp[axis] = eps;
            let p1 = Pose::new(pose.position + dp, pose.orientation);
            let l1 = lengths_at(&p1);
            for i in 0..4 {
                let num = (l1[i] - l0[i]) / eps;
                assert_relative_eq!(j[(i, axis)], num, epsilon = 1e-5);
            }
        }
        for axis in 0..3 {
            let mut dw = Vec3::zeros();
            dw[axis] = eps;
            let p_plus = Pose::new(
                pose.position,
                pose.orientation * crate::types::UnitQuat::from_scaled_axis(dw),
            );
            dw[axis] = -eps;
            let p_minus = Pose::new(
                pose.position,
                pose.orientation * crate::types::UnitQuat::from_scaled_axis(dw),
            );
            let l_plus = lengths_at(&p_plus);
            let l_minus = lengths_at(&p_minus);
            for i in 0..4 {
                let num = (l_plus[i] - l_minus[i]) / (2.0 * eps);
                assert_relative_eq!(j[(i, 3 + axis)], num, epsilon = 1e-6);
            }
        }
    }
}
