//! Venue mutation helpers.

use spyder_core::{Anchor, PlatformAttachment, Preset, Robot, Vec3};

use crate::dto::{FromPresetRequest, SetAnchorsRequest, VenueResponse};
use crate::state::{apply_cable_model, classify_robot, venue_from_state, AppState};
use crate::toml_venue::{emit_venue_toml, parse_venue_toml};

/// Load venue from TOML text.
pub async fn load_venue(state: &AppState, toml: &str) -> Result<VenueResponse, String> {
    let (robot, home) = parse_venue_toml(toml)?;
    {
        let mut r = state.robot.lock().await;
        *r = robot;
    }
    {
        let mut h = state.home.lock().await;
        *h = home;
    }
    venue_response(state).await
}

/// Build venue from a named preset.
pub async fn from_preset(
    state: &AppState,
    req: &FromPresetRequest,
) -> Result<VenueResponse, String> {
    let robot = match req.kind.as_str() {
        "rect" => Robot::from_preset(Preset::Rect {
            width: req.width.unwrap_or(10.0),
            depth: req.depth.unwrap_or(6.0),
            height: req.height.unwrap_or(8.0),
        })
        .map_err(|e| e.to_string())?,
        "polygon" => Robot::from_preset(Preset::RegularPolygon {
            n: req.n.unwrap_or(6),
            radius: req.radius.unwrap_or(5.0),
            height: req.height.unwrap_or(8.0),
        })
        .map_err(|e| e.to_string())?,
        other => return Err(format!("unknown preset kind: {other}")),
    };
    let mut robot = robot;
    robot.point_mass = req.point_mass;
    {
        let mut r = state.robot.lock().await;
        *r = robot;
    }
    venue_response(state).await
}

/// Replace anchors (and optional attachments).
pub async fn set_anchors(
    state: &AppState,
    req: &SetAnchorsRequest,
) -> Result<VenueResponse, String> {
    if req.anchors.len() < 3 {
        return Err("need at least 3 anchors".into());
    }
    let anchors: Vec<Anchor> = req
        .anchors
        .iter()
        .map(|a| Anchor::point(a.clone().into()))
        .collect();
    let attachments = req.attachments.as_ref().map(|atts| {
        atts.iter()
            .map(|a| PlatformAttachment::at(a.clone().into()))
            .collect::<Vec<_>>()
    });
    let robot = Robot::from_anchors(anchors, attachments, req.point_mass)
        .map_err(|e| e.to_string())?;
    let mut robot = robot;
    if let Some(ref model) = req.model {
        apply_cable_model(&mut robot, model)?;
    }
    {
        let mut r = state.robot.lock().await;
        *r = robot;
    }
    venue_response(state).await
}

/// Serialize current venue to TOML.
pub async fn venue_toml(state: &AppState) -> Result<String, String> {
    let robot = state.robot.lock().await;
    let home = *state.home.lock().await;
    let anchors: Vec<Vec3> = robot.anchors.iter().map(|a| a.exit).collect();
    let attachments: Vec<Vec3> = robot.attachments.iter().map(|a| a.body_point).collect();
    emit_venue_toml(&anchors, &attachments, robot.point_mass, home)
}

async fn venue_response(state: &AppState) -> Result<VenueResponse, String> {
    let venue = venue_from_state(state).await;
    let robot = state.robot.lock().await;
    let classify = classify_robot(&robot)?;
    Ok(VenueResponse { venue, classify })
}
