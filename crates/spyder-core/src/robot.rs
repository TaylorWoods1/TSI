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

    /// Restraint class from cable count vs DOF (3 point-mass / 6 platform).
    pub fn classify(&self) -> Result<spyder_statics::RestraintClass> {
        let n = if self.point_mass { 3 } else { 6 };
        spyder_statics::classify_restraint(self.anchors.len(), n)
            .map_err(SpyderError::Config)
    }

    /// Translational length Jacobian \(J\) with \(\dot L \approx J v\).
    pub fn length_jacobian(&self, pose: &Pose) -> Result<nalgebra::DMatrix<f64>> {
        let attachments = self.effective_attachments();
        crate::jacobian::length_jacobian(&self.anchors, &attachments, pose)
    }

    /// Forward kinematics (ideal length measurements).
    ///
    /// Point-mass: analytic fast paths when available, else numeric 3DOF.
    /// Platform: numeric 6DOF (position + orientation) seeded at `seed` with
    /// identity orientation — prefer [`Self::fk_with_seed`] when orientation is known.
    pub fn fk(&self, lengths: &[f64], seed: Vec3) -> Result<FkResult> {
        self.fk_with_seed(lengths, &Pose::from_position(seed))
    }

    /// FK with a full pose seed (important for platform orientation).
    pub fn fk_with_seed(&self, lengths: &[f64], seed: &Pose) -> Result<FkResult> {
        if !self.point_mass {
            return crate::fk::fk_platform_numeric(
                &self.anchors,
                &self.attachments,
                lengths,
                seed,
            );
        }
        let exits: Vec<Vec3> = self.anchors.iter().map(|a| a.exit).collect();
        if exits.len() == 3 {
            return fk_analytic_3(
                exits[0],
                exits[1],
                exits[2],
                lengths[0],
                lengths[1],
                lengths[2],
                seed.position,
            );
        }
        if is_axis_aligned_rect4(&exits) {
            return fk_analytic_rect4(&exits, lengths, seed.position);
        }
        fk_point_mass_numeric(&exits, lengths, seed.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::PlatformAttachment;
    use crate::ik::IkOptions;
    use crate::types::UnitQuat;
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

    #[test]
    fn classify_rect4_rrpm_triangle_crpm() {
        let rect = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        assert_eq!(
            rect.classify().unwrap(),
            spyder_statics::RestraintClass::Rrpm
        );
        let tri = Robot::from_preset(Preset::RegularPolygon {
            n: 3,
            radius: 2.0,
            height: 3.0,
        })
        .unwrap();
        assert_eq!(
            tri.classify().unwrap(),
            spyder_statics::RestraintClass::Crpm
        );
    }

    #[test]
    fn lengths_invariant_under_rigid_world_translation() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 6.0,
            depth: 4.0,
            height: 5.0,
        })
        .unwrap();
        let pose = Pose::from_position(Vec3::new(0.3, -0.2, 1.2));
        let l0 = robot.ik(&pose).unwrap().lengths;
        let shift = Vec3::new(10.0, -4.0, 2.5);
        let mut shifted = robot.clone();
        for a in &mut shifted.anchors {
            a.exit += shift;
        }
        let pose2 = Pose::from_position(pose.position + shift);
        let l1 = shifted.ik(&pose2).unwrap().lengths;
        for (a, b) in l0.iter().zip(l1.iter()) {
            assert_relative_eq!(a, b, epsilon = 1e-9);
        }
    }

    #[test]
    fn polygon_n_motors_ik_fk_round_trip() {
        for n in [3usize, 5, 6] {
            let robot = Robot::from_preset(Preset::RegularPolygon {
                n,
                radius: 3.0,
                height: 4.0,
            })
            .unwrap();
            let pose = Pose::from_position(Vec3::new(0.15, -0.1, 1.0));
            let ik = robot.ik(&pose).unwrap();
            assert_eq!(ik.lengths.len(), n);
            let fk = robot.fk(&ik.lengths, Vec3::new(0.0, 0.0, 1.5)).unwrap();
            assert_relative_eq!(fk.position.x, pose.position.x, epsilon = 1e-5);
            assert_relative_eq!(fk.position.y, pose.position.y, epsilon = 1e-5);
            assert_relative_eq!(fk.position.z, pose.position.z, epsilon = 1e-5);
        }
    }

    #[test]
    fn length_jacobian_rows_match_cable_count() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let j = robot
            .length_jacobian(&Pose::from_position(Vec3::new(0.0, 0.0, 1.0)))
            .unwrap();
        assert_eq!((j.nrows(), j.ncols()), (4, 3));
    }

    #[test]
    fn platform_6dof_ik_fk_round_trip() {
        let anchors = rect(6.0, 4.0, 5.0).unwrap();
        let offsets: Vec<_> = [
            Vec3::new(0.2, 0.15, 0.0),
            Vec3::new(-0.2, 0.15, 0.0),
            Vec3::new(-0.2, -0.15, 0.0),
            Vec3::new(0.2, -0.15, 0.0),
            Vec3::new(0.0, 0.25, 0.05),
            Vec3::new(0.0, -0.25, 0.05),
        ]
        .into_iter()
        .map(PlatformAttachment::at)
        .collect();
        // 6 cables for full 6DOF observability: extend rect with two mid-side anchors
        let mut anchors6 = anchors;
        anchors6.push(crate::anchor::Anchor::point(Vec3::new(0.0, 3.0, 5.0)));
        anchors6.push(crate::anchor::Anchor::point(Vec3::new(0.0, -3.0, 5.0)));
        let robot = Robot::from_anchors(anchors6, Some(offsets), false).unwrap();
        let orient = UnitQuat::from_scaled_axis(Vec3::new(0.05, -0.04, 0.08));
        let pose = Pose::new(Vec3::new(0.15, -0.1, 1.5), orient);
        let ik = robot.ik(&pose).unwrap();
        let seed = Pose::new(
            Vec3::new(0.0, 0.0, 1.6),
            UnitQuat::from_scaled_axis(Vec3::new(0.02, 0.0, 0.03)),
        );
        let fk = robot.fk_with_seed(&ik.lengths, &seed).unwrap();
        assert_eq!(fk.method, crate::fk::FkMethod::NumericPlatform6);
        assert_relative_eq!(fk.position.x, pose.position.x, epsilon = 1e-4);
        assert_relative_eq!(fk.position.y, pose.position.y, epsilon = 1e-4);
        assert_relative_eq!(fk.position.z, pose.position.z, epsilon = 1e-4);
        let q_err = (fk.orientation.inverse() * pose.orientation).scaled_axis().norm();
        assert!(q_err < 1e-3, "orientation error {q_err}");
        assert!(fk.residual < 1e-6);
    }
}
