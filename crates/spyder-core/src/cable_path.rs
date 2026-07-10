//! Model-aware cable polylines for visualization (GUI, Plotly export).

use spyder_cables::{CableContext, Pulley};

use crate::anchor::Anchor;
use crate::cable_eval::default_pulley_radius;
use crate::error::{Result, SpyderError};
use crate::robot::CableModelKind;
use crate::types::Vec3;

const PATH_SEGMENTS: usize = 12;

/// Build render polyline for one cable at world attachment `b`.
pub fn cable_path_vertices(
    anchor: &Anchor,
    b: &Vec3,
    model: &CableModelKind,
    ctx: &CableContext,
    default_pulley_radius: f64,
) -> Result<Vec<Vec3>> {
    let pts = match model {
        CableModelKind::Ideal => {
            vec![anchor.exit, *b]
        }
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
            let input = spyder_cables::pulley_geom::PulleyGeomInput {
                center: anchor.exit,
                axis: pulley.axis,
                radius: pulley.radius,
                winch_exit: pulley.winch_exit,
                runout_m: pulley.runout_m,
            };
            spyder_cables::pulley_geom::pulley_visual_polyline(b, &input, PATH_SEGMENTS)
                .map_err(|e| SpyderError::Model(e.to_string()))?
        }
        CableModelKind::Sag(sag) => {
            let tension = ctx.tension.unwrap_or(50.0);
            spyder_cables::sag_geom::sag_visual_polyline(sag, &anchor.exit, b, tension, PATH_SEGMENTS)
                .map_err(|e| SpyderError::Model(e.to_string()))?
        }
    };
    Ok(pts)
}

/// All cable paths at a pose for the active robot model.
pub fn cable_paths_at_pose(
    anchors: &[Anchor],
    attachments: &[crate::anchor::PlatformAttachment],
    pose: &crate::pose::Pose,
    model: &CableModelKind,
    tensions: Option<&[f64]>,
) -> Result<Vec<Vec<[f64; 3]>>> {
    if anchors.len() != attachments.len() {
        return Err(SpyderError::Config(
            "anchors/attachments length mismatch".into(),
        ));
    }
    let def_r = default_pulley_radius(model);
    let mut out = Vec::with_capacity(anchors.len());
    for (i, (anchor, att)) in anchors.iter().zip(attachments.iter()).enumerate() {
        let b = pose.transform_point(&att.body_point);
        let ctx = CableContext {
            tension: tensions.and_then(|t| t.get(i).copied()),
        };
        let path = cable_path_vertices(anchor, &b, model, &ctx, def_r)?;
        out.push(path.iter().map(|p| [p.x, p.y, p.z]).collect());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::PlatformAttachment;
    use crate::pose::Pose;
    use crate::preset::rect;
    use crate::robot::Robot;

    #[test]
    fn pulley_path_changes_with_height() {
        let mut anchors = rect(6.0, 4.0, 5.0).unwrap();
        for a in &mut anchors {
            a.pulley_axis = Some(Vec3::z());
            a.pulley_radius = 0.06;
        }
        let model = CableModelKind::Pulley {
            default_radius: 0.06,
        };
        let att = PlatformAttachment::origin();
        let ctx = CableContext::default();
        let low = cable_path_vertices(
            &anchors[0],
            &Vec3::new(0.3, -0.2, 1.0),
            &model,
            &ctx,
            0.06,
        )
        .unwrap();
        let high = cable_path_vertices(
            &anchors[0],
            &Vec3::new(0.3, -0.2, 2.0),
            &model,
            &ctx,
            0.06,
        )
        .unwrap();
        let len_low: f64 = low.windows(2).map(|w| (w[1] - w[0]).norm()).sum();
        let len_high: f64 = high.windows(2).map(|w| (w[1] - w[0]).norm()).sum();
        assert!((len_low - len_high).abs() > 1e-3);
    }

    #[test]
    fn robot_cable_paths_match_cable_count() {
        let robot = Robot::from_preset(crate::robot::Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let pose = Pose::from_position(Vec3::new(0.2, -0.1, 1.0));
        let paths = cable_paths_at_pose(
            &robot.anchors,
            &robot.attachments,
            &pose,
            &robot.cable_model,
            None,
        )
        .unwrap();
        assert_eq!(paths.len(), 4);
        assert!(paths[0].len() >= 2);
    }
}
