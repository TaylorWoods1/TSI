//! Inverse kinematics: pose → cable lengths.

use spyder_cables::{CableContext, CableModel, Ideal};

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
}
