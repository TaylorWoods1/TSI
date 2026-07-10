//! High-level robot facade combining anchors, attachments, and solves.

use spyder_cables::{CableContext, CableModel, Pulley, Sag};

use crate::anchor::{Anchor, PlatformAttachment};
use crate::cable_eval::{default_pulley_radius, predicted_lengths, unit_pulls_at_pose};
use crate::error::{Result, SpyderError};
use crate::fk::{fk_point_mass_from_anchors, fk_platform_numeric, FkOptions, FkResult};
use crate::fk_analytic::{fk_analytic_3, fk_analytic_rect4, is_axis_aligned_rect4};
use crate::ik::{ik_ideal, IkResult};
use crate::pose::Pose;
use crate::preset::{rect, regular_polygon};
use crate::types::Vec3;
use nalgebra::DVector;
use spyder_statics::{structure_matrix_3, structure_matrix_6, structure_rank, solve_tensions};

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
        if point_mass {
            for (i, att) in attachments.iter().enumerate() {
                if att.body_point.norm() > 1e-9 {
                    return Err(SpyderError::Config(format!(
                        "point_mass mode ignores attachment offset at index {i}; \
                         use point_mass=false for offset attachments"
                    )));
                }
            }
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
                return self.ik_sag(pose, &attachments, sag, opts);
            }
        };
        crate::ik::apply_ik_options(
            res,
            &self.anchors,
            &attachments,
            pose,
            self.point_mass,
            &self.cable_model,
            opts,
        )
    }

    fn pulley_for_anchor(&self, anchor: &Anchor, default_radius: f64) -> Result<Pulley> {
        let radius = if anchor.pulley_radius > 0.0 {
            anchor.pulley_radius
        } else {
            default_radius
        };
        let axis = anchor.pulley_axis.unwrap_or_else(Vec3::z);
        let mut pulley = Pulley::new(axis, radius).map_err(|e| SpyderError::Model(e.to_string()))?;
        if let Some(w) = anchor.pulley_winch_exit {
            pulley = pulley.with_winch_exit(w);
        }
        if anchor.pulley_runout_m > 0.0 {
            pulley = pulley.with_runout(anchor.pulley_runout_m);
        }
        Ok(pulley)
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
            let pulley = self.pulley_for_anchor(anchor, default_radius)?;
            let g = pulley
                .geometry(&anchor.exit, &b, &ctx)
                .map_err(|e| SpyderError::Model(e.to_string()))?;
            lengths.push(g.geometric);
            unstrained.push(g.unstrained);
        }
        Ok(IkResult {
            lengths,
            unstrained_lengths: unstrained,
            tensions: None,
            motor_commands: None,
        })
    }

    fn solve_tensions_at_pose(
        &self,
        pose: &Pose,
        attachments: &[PlatformAttachment],
        wrench: &DVector<f64>,
        f_min: f64,
        f_max: f64,
        tensions_hint: Option<&[f64]>,
    ) -> Result<Vec<f64>> {
        let def_r = default_pulley_radius(&self.cable_model);
        let unit_pulls =
            unit_pulls_at_pose(&self.anchors, attachments, pose, &self.cable_model, tensions_hint, def_r)?;
        let moment_arms: Vec<_> = attachments
            .iter()
            .map(|att| pose.transform_point(&att.body_point) - pose.position)
            .collect();
        let f = if self.point_mass {
            let a = structure_matrix_3(&unit_pulls).map_err(|e| SpyderError::Config(e.to_string()))?;
            let w = if wrench.len() == 3 {
                wrench.clone()
            } else {
                DVector::from_vec(vec![wrench[0], wrench[1], wrench[2]])
            };
            solve_tensions(&a, &w, f_min, f_max)
        } else {
            let a = structure_matrix_6(&unit_pulls, &moment_arms)
                .map_err(|e| SpyderError::Config(e.to_string()))?;
            let w = match wrench.len() {
                3 => DVector::from_vec(vec![wrench[0], wrench[1], wrench[2], 0.0, 0.0, 0.0]),
                6 => wrench.clone(),
                _ => {
                    return Err(SpyderError::Config(
                        "platform wrench must be 3-vector or 6-vector".into(),
                    ));
                }
            };
            solve_tensions(&a, &w, f_min, f_max)
        }
        .map_err(|e| match e {
            spyder_statics::TensionError::Infeasible => SpyderError::InfeasibleWrench,
            spyder_statics::TensionError::Singular => SpyderError::SingularStructure,
            other => SpyderError::Config(other.to_string()),
        })?;
        Ok(f.iter().copied().collect())
    }

    fn ik_sag(
        &self,
        pose: &Pose,
        attachments: &[PlatformAttachment],
        sag: &Sag,
        opts: &crate::ik::IkOptions,
    ) -> Result<IkResult> {
        let wrench = opts.wrench.as_ref().ok_or_else(|| {
            SpyderError::Config(
                "sag model requires IkOptions.wrench to estimate per-cable tension".into(),
            )
        })?;
        let f_min = if opts.f_min > 0.0 { opts.f_min } else { 1.0 };
        let f_max = if opts.f_max > f_min { opts.f_max } else { 1.0e4 };

        let mut tensions = self.solve_tensions_at_pose(pose, attachments, wrench, f_min, f_max, None)?;
        let max_iter = 12;
        let tol = 1e-4;

        let mut lengths = Vec::with_capacity(self.anchors.len());
        let mut unstrained = Vec::with_capacity(self.anchors.len());

        for _ in 0..max_iter {
            for (i, (anchor, att)) in self.anchors.iter().zip(attachments.iter()).enumerate() {
                let b = pose.transform_point(&att.body_point);
                let g = sag
                    .geometry(&anchor.exit, &b, &CableContext { tension: Some(tensions[i]) })
                    .map_err(|e| SpyderError::Model(e.to_string()))?;
                if i < lengths.len() {
                    lengths[i] = g.geometric;
                    unstrained[i] = g.unstrained;
                } else {
                    lengths.push(g.geometric);
                    unstrained.push(g.unstrained);
                }
            }
            let new_t = self.solve_tensions_at_pose(
                pose,
                attachments,
                wrench,
                f_min,
                f_max,
                Some(&tensions),
            )?;
            let delta: f64 = new_t
                .iter()
                .zip(tensions.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            tensions = new_t;
            if delta < tol {
                break;
            }
        }

        let result = IkResult {
            lengths,
            unstrained_lengths: unstrained,
            tensions: Some(tensions),
            motor_commands: None,
        };
        let motor_opts = crate::ik::IkOptions {
            wrench: None,
            f_min: opts.f_min,
            f_max: opts.f_max,
            reference_lengths: opts.reference_lengths.clone(),
            winches: opts.winches.clone(),
            motors: opts.motors.clone(),
        };
        crate::ik::apply_ik_options(
            result,
            &self.anchors,
            attachments,
            pose,
            self.point_mass,
            &self.cable_model,
            &motor_opts,
        )
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
        match self.ik_with_options(pose, &opts) {
            Ok(r) => Ok(r.tensions.is_some()),
            Err(SpyderError::InfeasibleWrench) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Restraint class from cable count vs DOF and structure-matrix rank.
    pub fn classify(&self) -> Result<spyder_statics::RestraintClass> {
        let n = if self.point_mass { 3 } else { 6 };
        let m = self.anchors.len();
        let pose = Pose::from_position(Vec3::zeros());
        let attachments = self.effective_attachments();
        let def_r = default_pulley_radius(&self.cable_model);
        let pulls = unit_pulls_at_pose(
            &self.anchors,
            &attachments,
            &pose,
            &self.cable_model,
            None,
            def_r,
        )?;
        let moment_arms: Vec<_> = attachments
            .iter()
            .map(|_| Vec3::zeros())
            .collect();
        let a = if self.point_mass {
            structure_matrix_3(&pulls).map_err(|e| SpyderError::Config(e.to_string()))?
        } else {
            structure_matrix_6(&pulls, &moment_arms)
                .map_err(|e| SpyderError::Config(e.to_string()))?
        };
        let rank = structure_rank(&a, 1e-8);
        spyder_statics::classify_restraint_ranked(m, n, rank).map_err(SpyderError::Config)
    }

    /// Translational length Jacobian \(J\) with \(\dot L \approx J v\).
    pub fn length_jacobian(&self, pose: &Pose) -> Result<nalgebra::DMatrix<f64>> {
        let attachments = self.effective_attachments();
        crate::jacobian::length_jacobian(&self.anchors, &attachments, pose)
    }

    /// Full 6-DOF platform length Jacobian with \(\dot L \approx J \xi\).
    pub fn length_jacobian_6(&self, pose: &Pose) -> Result<nalgebra::DMatrix<f64>> {
        let attachments = self.effective_attachments();
        let def_r = default_pulley_radius(&self.cable_model);
        let pulls = unit_pulls_at_pose(
            &self.anchors,
            &attachments,
            pose,
            &self.cable_model,
            None,
            def_r,
        )?;
        crate::jacobian::length_jacobian_platform_6_with_pulls(
            &self.anchors,
            &attachments,
            pose,
            &pulls,
        )
    }

    /// Forward kinematics using the configured cable model.
    pub fn fk(&self, lengths: &[f64], seed: Vec3) -> Result<FkResult> {
        self.fk_with_seed(lengths, &Pose::from_position(seed))
    }

    /// FK with a full pose seed (important for platform orientation).
    pub fn fk_with_seed(&self, lengths: &[f64], seed: &Pose) -> Result<FkResult> {
        self.fk_with_options(lengths, seed, &FkOptions::default())
    }

    /// FK with explicit options (underconstrained override, sag tensions).
    pub fn fk_with_options(
        &self,
        lengths: &[f64],
        seed: &Pose,
        opts: &FkOptions,
    ) -> Result<FkResult> {
        if !self.point_mass {
            return fk_platform_numeric(
                &self.anchors,
                &self.attachments,
                lengths,
                seed,
                &self.cable_model,
                opts,
            );
        }
        let exits: Vec<Vec3> = self.anchors.iter().map(|a| a.exit).collect();
        if matches!(self.cable_model, CableModelKind::Ideal) {
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
        }
        fk_point_mass_from_anchors(
            &self.anchors,
            lengths,
            seed.position,
            &self.cable_model,
            opts,
        )
    }

    /// Predict cable lengths at a pose for the active cable model.
    pub fn predicted_lengths_at(&self, pose: &Pose, tensions: Option<&[f64]>) -> Result<Vec<f64>> {
        let attachments = self.effective_attachments();
        predicted_lengths(
            &self.anchors,
            &attachments,
            pose,
            &self.cable_model,
            tensions,
            default_pulley_radius(&self.cable_model),
        )
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
        // Off-center pose: pulley path exceeds ideal chord.
        let pose2 = Pose::from_position(Vec3::new(0.5, 0.3, 1.0));
        let li2 = ideal.ik(&pose2).unwrap();
        let lp2 = pulley.ik(&pose2).unwrap();
        assert!(
            lp2.lengths.iter().zip(li2.lengths.iter()).any(|(p, i)| *p > *i + 1e-4),
            "expected pulley > ideal at off-center pose"
        );
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

    #[test]
    fn from_anchors_rejects_too_few_cables() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let two: Vec<_> = anchors.into_iter().take(2).collect();
        assert!(Robot::from_anchors(two, None, true).is_err());
    }

    #[test]
    fn pulley_ik_fk_round_trip() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 6.0,
            depth: 4.0,
            height: 5.0,
        })
        .unwrap()
        .with_cable_model(CableModelKind::Pulley {
            default_radius: 0.06,
        });
        let pose = Pose::from_position(Vec3::new(0.3, -0.2, 1.2));
        let ik = robot.ik(&pose).unwrap();
        let fk = robot.fk(&ik.lengths, Vec3::new(0.0, 0.0, 2.0)).unwrap();
        assert_relative_eq!(fk.position.x, pose.position.x, epsilon = 1e-4);
        assert_relative_eq!(fk.position.y, pose.position.y, epsilon = 1e-4);
        assert_relative_eq!(fk.position.z, pose.position.z, epsilon = 1e-4);
    }

    #[test]
    fn sag_without_wrench_errors() {
        use spyder_cables::Sag;
        let mut robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        robot.cable_model = CableModelKind::Sag(Sag::default());
        let pose = Pose::from_position(Vec3::new(0.0, 0.0, 1.5));
        assert!(robot.ik(&pose).is_err());
    }
}
