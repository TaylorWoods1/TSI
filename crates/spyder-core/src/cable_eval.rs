//! Model-aware cable geometry evaluation for IK, FK, statics, and Jacobians.

use spyder_cables::{CableContext, CableGeometry, CableModel, Ideal, Pulley};

use crate::anchor::{Anchor, PlatformAttachment};
use crate::error::{Result, SpyderError};
use crate::pose::Pose;
use crate::robot::CableModelKind;
use crate::types::Vec3;

/// Evaluate one cable's geometry at world attachment `b`.
pub fn cable_geometry_at(
    anchor: &Anchor,
    b: &Vec3,
    model: &CableModelKind,
    ctx: &CableContext,
    _default_pulley_radius: f64,
) -> Result<CableGeometry> {
    let g = match model {
        CableModelKind::Ideal => Ideal.geometry(&anchor.exit, b, ctx),
        CableModelKind::Pulley { default_radius } => {
            let radius = if anchor.pulley_radius > 0.0 {
                anchor.pulley_radius
            } else {
                *default_radius
            };
            let axis = anchor.pulley_axis.unwrap_or_else(Vec3::z);
            let mut pulley = Pulley::new(axis, radius).map_err(|e| SpyderError::Model(e.to_string()))?;
            if let Some(w) = anchor.pulley_winch_exit {
                pulley = pulley.with_winch_exit(w);
            }
            if anchor.pulley_runout_m > 0.0 {
                pulley = pulley.with_runout(anchor.pulley_runout_m);
            }
            pulley.geometry(&anchor.exit, b, ctx)
        }
        CableModelKind::Sag(sag) => {
            if ctx.tension.is_none() {
                Ideal.geometry(&anchor.exit, b, ctx)
            } else {
                sag.geometry(&anchor.exit, b, ctx)
            }
        }
    }
    .map_err(|e| SpyderError::Model(e.to_string()))?;
    Ok(g)
}

/// Predicted geometric cable lengths at `pose` for all cables.
pub fn predicted_lengths(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
    model: &CableModelKind,
    tensions: Option<&[f64]>,
    default_pulley_radius: f64,
) -> Result<Vec<f64>> {
    if anchors.len() != attachments.len() {
        return Err(SpyderError::Config(
            "anchors/attachments length mismatch".into(),
        ));
    }
    let mut out = Vec::with_capacity(anchors.len());
    for (i, (anchor, att)) in anchors.iter().zip(attachments.iter()).enumerate() {
        let b = pose.transform_point(&att.body_point);
        let ctx = CableContext {
            tension: tensions.and_then(|t| t.get(i).copied()),
        };
        let g = cable_geometry_at(anchor, &b, model, &ctx, default_pulley_radius)?;
        out.push(g.geometric);
    }
    Ok(out)
}

/// Unit pull directions at each attachment for statics / Jacobian.
pub fn unit_pulls_at_pose(
    anchors: &[Anchor],
    attachments: &[PlatformAttachment],
    pose: &Pose,
    model: &CableModelKind,
    tensions: Option<&[f64]>,
    default_pulley_radius: f64,
) -> Result<Vec<Vec3>> {
    if anchors.len() != attachments.len() {
        return Err(SpyderError::Config(
            "anchors/attachments length mismatch".into(),
        ));
    }
    let mut out = Vec::with_capacity(anchors.len());
    for (i, (anchor, att)) in anchors.iter().zip(attachments.iter()).enumerate() {
        let b = pose.transform_point(&att.body_point);
        let ctx = CableContext {
            tension: tensions.and_then(|t| t.get(i).copied()),
        };
        let g = cable_geometry_at(anchor, &b, model, &ctx, default_pulley_radius)?;
        out.push(g.unit_pull);
    }
    Ok(out)
}

/// Default pulley radius from cable model kind.
pub fn default_pulley_radius(model: &CableModelKind) -> f64 {
    match model {
        CableModelKind::Pulley { default_radius } => *default_radius,
        _ => 0.0,
    }
}
