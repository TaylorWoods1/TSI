//! High-level robot facade combining anchors, attachments, and solves.

use crate::anchor::{Anchor, PlatformAttachment};
use crate::error::{Result, SpyderError};
use crate::fk::{fk_point_mass_numeric, FkResult};
use crate::fk_analytic::{fk_analytic_3, fk_analytic_rect4, is_axis_aligned_rect4};
use crate::ik::{ik_ideal, IkResult};
use crate::pose::Pose;
use crate::preset::{rect, regular_polygon};
use crate::types::Vec3;

/// Named layout presets accepted by [`Robot::from_preset`].
#[derive(Clone, Debug)]
pub enum Preset {
    /// Axis-aligned rectangle (`n = 4`).
    Rect {
        /// Width along X (meters).
        width: f64,
        /// Depth along Y (meters).
        depth: f64,
        /// Anchor height Z (meters).
        height: f64,
    },
    /// Regular polygon with `n` sides.
    RegularPolygon {
        /// Number of motors / cables (>= 3).
        n: usize,
        /// Circumradius in XY (meters).
        radius: f64,
        /// Anchor height Z (meters).
        height: f64,
    },
}

/// Parametric cable robot configuration and solvers.
#[derive(Clone, Debug)]
pub struct Robot {
    /// Base exit points.
    pub anchors: Vec<Anchor>,
    /// Platform attachment points (body frame).
    pub attachments: Vec<PlatformAttachment>,
    /// When true, attachments are treated as coincident at the origin.
    pub point_mass: bool,
}

impl Robot {
    /// Build from a layout preset with coincident (point-mass) attachments.
    pub fn from_preset(preset: Preset) -> Result<Self> {
        let anchors = match preset {
            Preset::Rect {
                width,
                depth,
                height,
            } => rect(width, depth, height)?,
            Preset::RegularPolygon { n, radius, height } => regular_polygon(n, radius, height)?,
        };
        Self::from_anchors(anchors, None, true)
    }

    /// Build from explicit anchors.
    ///
    /// If `attachments` is `None`, coincident origins are used.
    /// `point_mass` forces body points to origin during IK regardless of attachments.
    pub fn from_anchors(
        anchors: Vec<Anchor>,
        attachments: Option<Vec<PlatformAttachment>>,
        point_mass: bool,
    ) -> Result<Self> {
        if anchors.len() < 3 {
            return Err(SpyderError::Config("need at least 3 anchors".into()));
        }
        let n = anchors.len();
        let attachments = attachments.unwrap_or_else(|| {
            (0..n).map(|_| PlatformAttachment::origin()).collect()
        });
        if attachments.len() != n {
            return Err(SpyderError::Config(
                "attachments must match anchor count".into(),
            ));
        }
        Ok(Self {
            anchors,
            attachments,
            point_mass,
        })
    }

    fn effective_attachments(&self) -> Vec<PlatformAttachment> {
        if self.point_mass {
            self.anchors
                .iter()
                .map(|_| PlatformAttachment::origin())
                .collect()
        } else {
            self.attachments.clone()
        }
    }

    /// Ideal-model inverse kinematics.
    pub fn ik(&self, pose: &Pose) -> Result<IkResult> {
        ik_ideal(&self.anchors, &self.effective_attachments(), pose)
    }

    /// Forward kinematics with automatic analytic dispatch when possible.
    pub fn fk(&self, lengths: &[f64], seed: Vec3) -> Result<FkResult> {
        if !self.point_mass {
            // Platform FK: optimize translation only for now (orientation fixed identity seed)
            // Full 6DOF FK lands with numeric extension; use point-mass numeric on transformed
            // coincident approximation when attachments are tiny — for nonzero offsets, solve
            // translation with fixed orientation = identity as Phase 1 subset.
            return self.fk_platform_translation(lengths, seed);
        }
        let exits: Vec<Vec3> = self.anchors.iter().map(|a| a.exit).collect();
        if exits.len() == 3 {
            return fk_analytic_3(
                exits[0], exits[1], exits[2], lengths[0], lengths[1], lengths[2], seed,
            );
        }
        if is_axis_aligned_rect4(&exits) {
            return fk_analytic_rect4(&exits, lengths, seed);
        }
        fk_point_mass_numeric(&exits, lengths, seed)
    }

    fn fk_platform_translation(&self, lengths: &[f64], seed: Vec3) -> Result<FkResult> {
        // Gauss-Newton on translation with fixed identity orientation.
        let mut p = seed;
        let max_iters = 50;
        let mut residual = f64::INFINITY;
        let mut iterations = 0;

        for iter in 0..max_iters {
            iterations = iter + 1;
            let mut jtj = nalgebra::Matrix3::zeros();
            let mut jtr = Vec3::zeros();
            residual = 0.0;
            let pose = Pose::from_position(p);
            for (i, (anchor, att)) in self
                .anchors
                .iter()
                .zip(self.attachments.iter())
                .enumerate()
            {
                let b = pose.transform_point(&att.body_point);
                let diff = b - anchor.exit;
                let dist = diff.norm();
                if dist <= f64::EPSILON {
                    return Err(SpyderError::Geometry(
                        "FK iterate coincides with anchor".into(),
                    ));
                }
                let err = dist - lengths[i];
                residual += err * err;
                // ∂||(p+Rb)-a||/∂p = unit vector (same as point-mass when R fixed)
                let u = diff / dist;
                jtj += u * u.transpose();
                jtr += u * err;
            }
            residual = residual.sqrt();
            let delta = nalgebra::linalg::SVD::new(jtj, true, true)
                .solve(&jtr, 1e-12)
                .map_err(|_| SpyderError::SingularStructure)?;
            p -= delta;
            if delta.norm() < 1e-12 || residual < 1e-10 {
                return Ok(FkResult {
                    position: p,
                    residual,
                    iterations,
                    method: crate::fk::FkMethod::NumericPointMass,
                });
            }
        }
        Err(SpyderError::FkNonConvergence {
            residual,
            iterations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::PlatformAttachment;
    use approx::assert_relative_eq;

    #[test]
    fn platform_mode_changes_lengths_vs_point_mass() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let offsets: Vec<_> = [
            Vec3::new(0.1, 0.1, 0.0),
            Vec3::new(-0.1, 0.1, 0.0),
            Vec3::new(-0.1, -0.1, 0.0),
            Vec3::new(0.1, -0.1, 0.0),
        ]
        .into_iter()
        .map(PlatformAttachment::at)
        .collect();

        let pm = Robot::from_anchors(anchors.clone(), None, true).unwrap();
        let plat = Robot::from_anchors(anchors, Some(offsets), false).unwrap();
        let pose = Pose::from_position(Vec3::new(0.0, 0.0, 1.0));
        let l_pm = pm.ik(&pose).unwrap();
        let l_plat = plat.ik(&pose).unwrap();
        assert!(
            l_pm
                .lengths
                .iter()
                .zip(l_plat.lengths.iter())
                .any(|(a, b)| (a - b).abs() > 1e-6),
            "platform offsets should change lengths"
        );
    }

    #[test]
    fn robot_ik_fk_round_trip() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 6.0,
            depth: 4.0,
            height: 5.0,
        })
        .unwrap();
        let pose = Pose::from_position(Vec3::new(0.4, -0.3, 1.5));
        let ik = robot.ik(&pose).unwrap();
        let fk = robot.fk(&ik.lengths, Vec3::new(0.0, 0.0, 2.0)).unwrap();
        assert_relative_eq!(fk.position.x, pose.position.x, epsilon = 1e-5);
        assert_relative_eq!(fk.position.y, pose.position.y, epsilon = 1e-5);
        assert_relative_eq!(fk.position.z, pose.position.z, epsilon = 1e-5);
    }
}
