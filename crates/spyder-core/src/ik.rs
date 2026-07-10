//! Inverse kinematics: pose → cable lengths.

use nalgebra::DVector;
use spyder_actuation::{length_delta_to_command, Motor, MotorCommand, Winch};
use spyder_cables::{CableContext, CableModel, Ideal};
use spyder_statics::{closed_form_tensions, structure_matrix_3, structure_matrix_6};

use crate::anchor::{Anchor, PlatformAttachment};
use crate::error::{Result, SpyderError};
use crate::pose::Pose;
use crate::types::Vec3;

/// Result of an inverse kinematics solve.
#[derive(Clone, Debug)]
pub struct IkResult {
    /// Geometric cable lengths (meters), one per cable.
    pub lengths: Vec<f64>,
    /// Optional unstrained lengths when the cable model provides them.
    pub unstrained_lengths: Vec<Option<f64>>,
    /// Optional cable tensions (Newtons) when a wrench was provided.
    pub tensions: Option<Vec<f64>>,
    /// Optional motor commands from length deltas vs `reference_lengths`.
    pub motor_commands: Option<Vec<MotorCommand>>,
}

/// Options for an IK solve.
#[derive(Clone, Debug, Default)]
pub struct IkOptions {
    /// External wrench in world frame. Point-mass: 3-vector force. Platform: 6-vector force+torque.
    pub wrench: Option<DVector<f64>>,
    /// Tension bounds (Newtons).
    pub f_min: f64,
    /// Tension upper bound.
    pub f_max: f64,
    /// Reference lengths for motor delta mapping (e.g. home pose lengths).
    pub reference_lengths: Option<Vec<f64>>,
    /// Winches aligned with cables (for motor mapping).
    pub winches: Option<Vec<Winch>>,
    /// Motors aligned with cables.
    pub motors: Option<Vec<Motor>>,
}

impl IkOptions {
    /// Default tension bounds.
    pub fn with_defaults() -> Self {
        Self {
            wrench: None,
            f_min: 1.0,
            f_max: 1.0e4,
            reference_lengths: None,
            winches: None,
            motors: None,
        }
    }
}

/// Ideal-model IK for a point-mass at `position` with world anchors.
pub fn ideal_ik_point_mass(anchors: &[Vec3], position: &Vec3) -> Result<Vec<f64>> {
    let model = Ideal;
    let ctx = CableContext::default();
    let mut lengths = Vec::with_capacity(anchors.len());
    for a in anchors {
        let len = model
            .length(a, position, &ctx)
            .map_err(|e| SpyderError::Model(e.to_string()))?;
        lengths.push(len.geometric);
    }
    Ok(lengths)
}

/// General IK: transform each platform attachment, then evaluate `model`.
pub fn ik_with_model<M: CableModel>(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
    model: &M,
    ctx: &CableContext,
) -> Result<IkResult> {
    if anchors.len() != attachments.len() {
        return Err(SpyderError::Config(
            "anchors and attachments length mismatch".into(),
        ));
    }
    if anchors.len() < 3 {
        return Err(SpyderError::Config("need at least 3 cables".into()));
    }

    let mut lengths = Vec::with_capacity(anchors.len());
    let mut unstrained = Vec::with_capacity(anchors.len());
    for (anchor, att) in anchors.iter().zip(attachments.iter()) {
        let b_world = pose.transform_point(&att.body_point);
        let len = model
            .length(&anchor.exit, &b_world, ctx)
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

/// Ideal IK convenience using [`Ideal`].
pub fn ik_ideal(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
) -> Result<IkResult> {
    ik_with_model(
        anchors,
        attachments,
        pose,
        &Ideal,
        &CableContext::default(),
    )
}

/// Enrich an [`IkResult`] with tensions and/or motor commands.
pub fn apply_ik_options(
    mut result: IkResult,
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
    point_mass: bool,
    opts: &IkOptions,
) -> Result<IkResult> {
    if let Some(ref wrench) = opts.wrench {
        let mut unit_pulls = Vec::with_capacity(anchors.len());
        let mut moment_arms = Vec::with_capacity(anchors.len());
        for (anchor, att) in anchors.iter().zip(attachments.iter()) {
            let b_world = pose.transform_point(&att.body_point);
            let diff = anchor.exit - b_world;
            let dist = diff.norm();
            if dist <= f64::EPSILON {
                return Err(SpyderError::Geometry(
                    "zero cable for structure matrix".into(),
                ));
            }
            let u = diff / dist;
            unit_pulls.push(u);
            // moment arm from platform origin to attachment in world
            moment_arms.push(b_world - pose.position);
        }
        let f_min = if opts.f_min > 0.0 { opts.f_min } else { 1.0 };
        let f_max = if opts.f_max > f_min {
            opts.f_max
        } else {
            1.0e4
        };
        let tensions = if point_mass || wrench.len() == 3 {
            let a = structure_matrix_3(&unit_pulls).map_err(|e| SpyderError::Config(e.to_string()))?;
            let w = if wrench.len() == 3 {
                wrench.clone()
            } else if wrench.len() == 6 {
                DVector::from_vec(vec![wrench[0], wrench[1], wrench[2]])
            } else {
                return Err(SpyderError::Config(
                    "point-mass wrench must be 3 or first-3 of 6".into(),
                ));
            };
            closed_form_tensions(&a, &w, f_min, f_max).map_err(|e| match e {
                spyder_statics::TensionError::Infeasible => SpyderError::InfeasibleWrench,
                spyder_statics::TensionError::Singular => SpyderError::SingularStructure,
                other => SpyderError::Config(other.to_string()),
            })?
        } else {
            let a = structure_matrix_6(&unit_pulls, &moment_arms)
                .map_err(|e| SpyderError::Config(e.to_string()))?;
            if wrench.len() != 6 {
                return Err(SpyderError::Config(
                    "platform wrench must be 6-vector".into(),
                ));
            }
            closed_form_tensions(&a, wrench, f_min, f_max).map_err(|e| match e {
                spyder_statics::TensionError::Infeasible => SpyderError::InfeasibleWrench,
                spyder_statics::TensionError::Singular => SpyderError::SingularStructure,
                other => SpyderError::Config(other.to_string()),
            })?
        };
        result.tensions = Some(tensions.iter().copied().collect());
    }

    if let (Some(refs), Some(winches), Some(motors)) =
        (&opts.reference_lengths, &opts.winches, &opts.motors)
    {
        if refs.len() != result.lengths.len()
            || winches.len() != result.lengths.len()
            || motors.len() != result.lengths.len()
        {
            return Err(SpyderError::Config(
                "reference_lengths/winches/motors must match cable count".into(),
            ));
        }
        let mut cmds = Vec::with_capacity(result.lengths.len());
        for i in 0..result.lengths.len() {
            let delta = result.lengths[i] - refs[i];
            cmds.push(length_delta_to_command(&winches[i], &motors[i], delta));
        }
        result.motor_commands = Some(cmds);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::PlatformAttachment;
    use crate::preset::rect;
    use approx::assert_relative_eq;

    #[test]
    fn ideal_ik_rect_center() {
        let exits = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
        ];
        let lengths = ideal_ik_point_mass(&exits, &Vec3::new(0.0, 0.0, 0.0)).unwrap();
        for l in &lengths {
            assert_relative_eq!(*l, 3f64.sqrt(), epsilon = 1e-9);
        }
    }

    #[test]
    fn ik_ideal_with_preset_attachments() {
        let anchors = rect(2.0, 2.0, 1.0).unwrap();
        let attachments: Vec<_> = (0..4).map(|_| PlatformAttachment::origin()).collect();
        let pose = Pose::from_position(Vec3::new(0.0, 0.0, 0.0));
        let res = ik_ideal(&anchors, &attachments, &pose).unwrap();
        assert_eq!(res.lengths.len(), 4);
        for l in &res.lengths {
            assert_relative_eq!(*l, 3f64.sqrt(), epsilon = 1e-9);
        }
    }

    #[test]
    fn ik_with_gravity_tensions() {
        let anchors = rect(2.0, 2.0, 1.0).unwrap();
        let attachments: Vec<_> = (0..4).map(|_| PlatformAttachment::origin()).collect();
        let pose = Pose::from_position(Vec3::new(0.0, 0.0, 0.0));
        let res = ik_ideal(&anchors, &attachments, &pose).unwrap();
        let opts = IkOptions {
            wrench: Some(DVector::from_vec(vec![0.0, 0.0, -9.81])),
            f_min: 0.1,
            f_max: 1.0e3,
            ..IkOptions::with_defaults()
        };
        let enriched = apply_ik_options(res, &anchors, &attachments, &pose, true, &opts).unwrap();
        let t = enriched.tensions.expect("tensions");
        assert_eq!(t.len(), 4);
        assert!(t.iter().all(|x| *x > 0.0));
    }
}
