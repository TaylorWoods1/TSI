//! High-level robot facade combining anchors, attachments, and solves.

use spyder_cables::{CableContext, CableModel, Pulley, Sag};

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

/// Which cable length model the robot uses for IK.
#[derive(Clone, Debug)]
pub enum CableModelKind {
    /// Straight Euclidean cables.
    Ideal,
    /// Swivel-pulley compensation (uses each anchor's axis/radius, or defaults).
    Pulley {
        /// Default radius when an anchor has `pulley_radius == 0`.
        default_radius: f64,
    },
    /// Irvine sag (requires tension via wrench in [`crate::ik::IkOptions`]).
    Sag(Sag),
}

impl Default for CableModelKind {
    fn default() -> Self {
        Self::Ideal
    }
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
    /// Active cable model.
    pub cable_model: CableModelKind,
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
    pub fn from_anchors(
        anchors: Vec<Anchor>,
        attachments: Option<Vec<PlatformAttachment>>,
        point_mass: bool,
    ) -> Result<Self> {
        if anchors.len() < 3 {
            return Err(SpyderError::Config("need at least 3 anchors".into()));
        }
        let n = anchors.len();
        let attachments = attachments
            .unwrap_or_else(|| (0..n).map(|_| PlatformAttachment::origin()).collect());
        if attachments.len() != n {
            return Err(SpyderError::Config(
                "attachments must match anchor count".into(),
            ));
        }
        Ok(Self {
            anchors,
            attachments,
            point_mass,
            cable_model: CableModelKind::Ideal,
        })
    }

    /// Set the cable model (builder-style).
    pub fn with_cable_model(mut self, model: CableModelKind) -> Self {
        self.cable_model = model;
        self
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

    /// Inverse kinematics using the configured cable model.
    pub fn ik(&self, pose: &Pose) -> Result<IkResult> {
        self.ik_with_options(pose, &crate::ik::IkOptions::with_defaults())
    }

    /// IK with tension / motor mapping options.
    pub fn ik_with_options(
        &self,
        pose: &Pose,
        opts: &crate::ik::IkOptions,
    ) -> Result<IkResult> {
        let attachments = self.effective_attachments();
        let res = match &self.cable_model {
            CableModelKind::Ideal => ik_ideal(&self.anchors, &attachments, pose)?,
            CableModelKind::Pulley { default_radius } => {
                self.ik_pulley(pose, &attachments, *default_radius)?
            }
            CableModelKind::Sag(sag) => {
                // sag path already embeds tensions; still allow motor mapping via apply
                let sag_res = self.ik_sag(pose, &attachments, sag, opts)?;
                let unstrained = sag_res.unstrained_lengths.clone();
                let mut out = crate::ik::apply_ik_options(
                    IkResult {
                        tensions: None,
                        motor_commands: None,
                        ..sag_res
                    },
                    &self.anchors,
                    &attachments,
                    pose,
                    self.point_mass,
                    opts,
                )?;
                out.unstrained_lengths = unstrained;
                return Ok(out);
            }
        };
        crate::ik::apply_ik_options(res, &self.anchors, &attachments, pose, self.point_mass, opts)
    }

    fn ik_pulley(
        &self,
        pose: &Pose,
        attachments: &[PlatformAttachment],
        default_radius: f64,
    ) -> Result<IkResult> {
        let mut lengths = Vec::with_capacity(self.anchors.len());
        let mut unstrained = Vec::with_capacity(self.anchors.len());
        let ctx = CableContext::default();
        for (anchor, att) in self.anchors.iter().zip(attachments.iter()) {
            let b = pose.transform_point(&att.body_point);
            let radius = if anchor.pulley_radius > 0.0 {
                anchor.pulley_radius
            } else {
                default_radius
            };
            let axis = anchor.pulley_axis.unwrap_or_else(Vec3::z);
            let model = Pulley::new(axis, radius).map_err(|e| SpyderError::Model(e.to_string()))?;
            let len = model
                .length(&anchor.exit, &b, &ctx)
                .map_err(|e| SpyderError::Model(e.to_string()))?;
            lengths.push(len.geometric);
            unstrained.push(len.unstrained);
        }
        Ok(IkResult {
            lengths,
            unstrained_lengths: unstrained,
            tensions: None,
            motor_commands: None,
        })
    }

    fn ik_sag(
        &self,
        pose: &Pose,
        attachments: &[PlatformAttachment],
        sag: &Sag,
        opts: &crate::ik::IkOptions,
    ) -> Result<IkResult> {
        let ideal = ik_ideal(&self.anchors, attachments, pose)?;
        let with_t = crate::ik::apply_ik_options(
            ideal,
            &self.anchors,
            attachments,
            pose,
            self.point_mass,
            opts,
        )?;
        let tensions = with_t.tensions.as_ref().ok_or_else(|| {
            SpyderError::Config(
                "sag model requires IkOptions.wrench to estimate per-cable tension".into(),
            )
        })?;

        let mut lengths = Vec::with_capacity(self.anchors.len());
        let mut unstrained = Vec::with_capacity(self.anchors.len());
        for (i, (anchor, att)) in self.anchors.iter().zip(attachments.iter()).enumerate() {
            let b = pose.transform_point(&att.body_point);
            let ctx = CableContext {
                tension: Some(tensions[i]),
            };
            let len = sag
                .length(&anchor.exit, &b, &ctx)
                .map_err(|e| SpyderError::Model(e.to_string()))?;
            lengths.push(len.geometric);
            unstrained.push(len.unstrained);
        }
        Ok(IkResult {
            lengths,
            unstrained_lengths: unstrained,
            tensions: Some(tensions.clone()),
            motor_commands: None,
        })
    }

    /// Wrench feasibility at a pose.
    pub fn is_wrench_feasible(
        &self,
        pose: &Pose,
        wrench: nalgebra::DVector<f64>,
        f_min: f64,
        f_max: f64,
    ) -> Result<bool> {
        let opts = crate::ik::IkOptions {
            wrench: Some(wrench),
            f_min,
            f_max,
            ..crate::ik::IkOptions::with_defaults()
        };
        let mut tmp = self.clone();
        tmp.cable_model = CableModelKind::Ideal;
        match tmp.ik_with_options(pose, &opts) {
            Ok(r) => Ok(r.tensions.is_some()),
            Err(SpyderError::InfeasibleWrench) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Forward kinematics (ideal length measurements).
    pub fn fk(&self, lengths: &[f64], seed: Vec3) -> Result<FkResult> {
        if !self.point_mass {
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
    use crate::ik::IkOptions;
    use approx::assert_relative_eq;
    use nalgebra::DVector;

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
        assert!(l_pm
            .lengths
            .iter()
            .zip(l_plat.lengths.iter())
            .any(|(a, b)| (a - b).abs() > 1e-6));
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

    #[test]
    fn pulley_model_increases_lengths() {
        let ideal = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let pulley = ideal.clone().with_cable_model(CableModelKind::Pulley {
            default_radius: 0.08,
        });
        let pose = Pose::from_position(Vec3::new(0.0, 0.0, 1.0));
        let li = ideal.ik(&pose).unwrap();
        let lp = pulley.ik(&pose).unwrap();
        for (a, b) in li.lengths.iter().zip(lp.lengths.iter()) {
            assert!(b > a, "pulley length {b} should exceed ideal {a}");
        }
    }

    #[test]
    fn sag_model_returns_unstrained_with_wrench() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 10.0,
            depth: 10.0,
            height: 8.0,
        })
        .unwrap()
        .with_cable_model(CableModelKind::Sag(Sag::default()));
        let pose = Pose::from_position(Vec3::new(0.0, 0.0, 2.0));
        let opts = IkOptions {
            wrench: Some(DVector::from_vec(vec![0.0, 0.0, -50.0])),
            f_min: 1.0,
            f_max: 1.0e4,
            ..IkOptions::with_defaults()
        };
        let ik = robot.ik_with_options(&pose, &opts).unwrap();
        assert!(ik.unstrained_lengths.iter().all(|u| u.is_some()));
        assert!(ik.tensions.is_some());
    }
}
