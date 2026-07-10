//! Venue mutation helpers.

use spyder_core::{Anchor, PlatformAttachment, Preset, Robot, Vec3};

use crate::dto::{FromPresetRequest, SetAnchorsRequest, SetCableModelRequest, VenueResponse};
use crate::state::{
    apply_cable_model, cable_model_params, classify_robot, venue_from_state, AppState,
    CableModelParams,
};
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

fn model_params_from_request(
    base: &CableModelParams,
    model: Option<&str>,
    pulley_radius: Option<f64>,
    sag_mu: Option<f64>,
    sag_ea: Option<f64>,
) -> CableModelParams {
    let mut params = base.clone();
    if let Some(m) = model {
        params.model = m.to_string();
    }
    if let Some(r) = pulley_radius {
        params.pulley_radius = r;
    }
    if let Some(mu) = sag_mu {
        params.sag_mu = mu;
    }
    if let Some(ea) = sag_ea {
        params.sag_ea = ea;
    }
    params
}

/// Replace anchors (and optional attachments).
pub async fn set_anchors(
    state: &AppState,
    req: &SetAnchorsRequest,
) -> Result<VenueResponse, String> {
    if req.anchors.len() < 3 {
        return Err("need at least 3 anchors".into());
    }
    let prev = state.robot.lock().await.clone();
    let params = model_params_from_request(
        &cable_model_params(&prev),
        req.model.as_deref(),
        req.pulley_radius,
        req.sag_mu,
        req.sag_ea,
    );
    let anchors: Vec<Anchor> = req
        .anchors
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let v: Vec3 = a.clone().into();
            if let Some(prev_a) = prev.anchors.get(i) {
                let mut anchor = prev_a.clone();
                anchor.exit = v;
                anchor
            } else if params.model == "pulley" {
                Anchor::with_z_pulley(v, params.pulley_radius)
            } else {
                Anchor::point(v)
            }
        })
        .collect();
    let attachments = req.attachments.as_ref().map(|atts| {
        atts.iter()
            .map(|a| PlatformAttachment::at(a.clone().into()))
            .collect::<Vec<_>>()
    });
    let robot = Robot::from_anchors(anchors, attachments, req.point_mass)
        .map_err(|e| e.to_string())?;
    let mut robot = robot;
    apply_cable_model(&mut robot, &params)?;
    {
        let mut r = state.robot.lock().await;
        *r = robot;
    }
    venue_response(state).await
}

/// Set cable model and parameters.
pub async fn set_cable_model(
    state: &AppState,
    req: &SetCableModelRequest,
) -> Result<VenueResponse, String> {
    let mut robot = state.robot.lock().await.clone();
    let params = model_params_from_request(
        &cable_model_params(&robot),
        Some(&req.model),
        req.pulley_radius,
        req.sag_mu,
        req.sag_ea,
    );
    apply_cable_model(&mut robot, &params)?;
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
    let params = cable_model_params(&robot);
    let anchors: Vec<Vec3> = robot.anchors.iter().map(|a| a.exit).collect();
    let attachments: Vec<Vec3> = robot.attachments.iter().map(|a| a.body_point).collect();
    emit_venue_toml(
        &anchors,
        &attachments,
        robot.point_mass,
        home,
        &params,
    )
}

async fn venue_response(state: &AppState) -> Result<VenueResponse, String> {
    let venue = venue_from_state(state).await;
    let robot = state.robot.lock().await;
    let classify = classify_robot(&robot)?;
    Ok(VenueResponse { venue, classify })
}
