//! Shared application state.

use std::sync::Arc;

use spyder_cables::Sag;
use spyder_core::{Preset, Robot, Vec3};
use tokio::sync::Mutex;

use crate::dto::VenueDto;
use crate::run_svc::RunSession;

/// Server-side robot and home pose.
pub struct AppState {
    /// Current robot configuration.
    pub robot: Mutex<Robot>,
    /// Home pose for calibration / playback.
    pub home: Mutex<Vec3>,
    /// Optional connected run session.
    pub run_session: Mutex<Option<RunSession>>,
}

impl AppState {
    /// Default rectangular 4-cable venue.
    pub fn new_rect() -> Arc<Self> {
        let robot = Robot::from_preset(Preset::Rect {
            width: 10.0,
            depth: 6.0,
            height: 8.0,
        })
        .expect("default robot");
        Arc::new(Self {
            robot: Mutex::new(robot),
            home: Mutex::new(Vec3::new(0.0, 0.0, 2.0)),
            run_session: Mutex::new(None),
        })
    }
}

/// Cable model parameters from the API.
#[derive(Clone, Debug, Default)]
pub struct CableModelParams {
    /// `ideal`, `pulley`, or `sag`.
    pub model: String,
    /// Default pulley radius (meters).
    pub pulley_radius: f64,
    /// Sag mass per unit length (kg/m).
    pub sag_mu: f64,
    /// Sag axial stiffness EA (N).
    pub sag_ea: f64,
}

/// Extract model parameters from a robot.
pub fn cable_model_params(robot: &Robot) -> CableModelParams {
    match &robot.cable_model {
        spyder_core::CableModelKind::Ideal => CableModelParams {
            model: "ideal".into(),
            pulley_radius: 0.05,
            sag_mu: 1.0,
            sag_ea: 1.0e6,
        },
        spyder_core::CableModelKind::Pulley { default_radius } => CableModelParams {
            model: "pulley".into(),
            pulley_radius: *default_radius,
            sag_mu: 1.0,
            sag_ea: 1.0e6,
        },
        spyder_core::CableModelKind::Sag(sag) => CableModelParams {
            model: "sag".into(),
            pulley_radius: 0.05,
            sag_mu: sag.mu,
            sag_ea: sag.ea,
        },
    }
}

/// Build a venue DTO from current state.
pub async fn venue_from_state(state: &AppState) -> VenueDto {
    let robot = state.robot.lock().await;
    let home = *state.home.lock().await;
    let params = cable_model_params(&robot);
    VenueDto {
        anchors: robot.anchors.iter().map(|a| a.exit.into()).collect(),
        attachments: robot
            .attachments
            .iter()
            .map(|a| a.body_point.into())
            .collect(),
        point_mass: robot.point_mass,
        model: params.model,
        pulley_radius: params.pulley_radius,
        sag_mu: params.sag_mu,
        sag_ea: params.sag_ea,
        home: home.into(),
    }
}

/// Convert cable model enum to API string.
pub fn cable_model_str(model: &spyder_core::CableModelKind) -> &'static str {
    match model {
        spyder_core::CableModelKind::Ideal => "ideal",
        spyder_core::CableModelKind::Pulley { .. } => "pulley",
        spyder_core::CableModelKind::Sag(_) => "sag",
    }
}

/// Apply cable model + parameters onto a robot and configure anchor pulleys.
pub fn apply_cable_model(robot: &mut Robot, params: &CableModelParams) -> Result<(), String> {
    robot.cable_model = match params.model.as_str() {
        "ideal" => spyder_core::CableModelKind::Ideal,
        "pulley" => spyder_core::CableModelKind::Pulley {
            default_radius: params.pulley_radius.max(1e-6),
        },
        "sag" => spyder_core::CableModelKind::Sag(Sag {
            mu: params.sag_mu.max(1e-6),
            ea: params.sag_ea.max(1.0),
            g: 9.81,
        }),
        other => return Err(format!("unknown cable model: {other}")),
    };
    if params.model == "pulley" {
        let r = params.pulley_radius.max(1e-6);
        for anchor in &mut robot.anchors {
            if anchor.pulley_axis.is_none() {
                anchor.pulley_axis = Some(Vec3::z());
            }
            if anchor.pulley_radius <= 0.0 {
                anchor.pulley_radius = r;
            }
        }
    }
    Ok(())
}

/// Parse API cable model string onto a robot (legacy helper).
pub fn apply_cable_model_str(robot: &mut Robot, model: &str) -> Result<(), String> {
    let mut params = cable_model_params(robot);
    params.model = model.to_string();
    apply_cable_model(robot, &params)
}

/// Classify the current robot restraint.
pub fn classify_robot(robot: &Robot) -> Result<String, String> {
    robot
        .classify()
        .map(|c| c.as_str().to_string())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_cable_model_parses_known_kinds() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let mut r = robot;
        apply_cable_model(
            &mut r,
            &CableModelParams {
                model: "pulley".into(),
                pulley_radius: 0.08,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(cable_model_str(&r.cable_model), "pulley");
        assert!(r.anchors[0].pulley_axis.is_some());
        assert!(apply_cable_model_str(&mut r, "nope").is_err());
    }
}
