//! Shared application state.

use std::sync::Arc;

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

/// Build a venue DTO from current state.
pub async fn venue_from_state(state: &AppState) -> VenueDto {
    let robot = state.robot.lock().await;
    let home = *state.home.lock().await;
    VenueDto {
        anchors: robot.anchors.iter().map(|a| a.exit.into()).collect(),
        attachments: robot
            .attachments
            .iter()
            .map(|a| a.body_point.into())
            .collect(),
        point_mass: robot.point_mass,
        model: cable_model_str(&robot.cable_model).into(),
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

/// Parse API cable model string onto a robot.
pub fn apply_cable_model(robot: &mut Robot, model: &str) -> Result<(), String> {
    robot.cable_model = match model {
        "ideal" => spyder_core::CableModelKind::Ideal,
        "pulley" => spyder_core::CableModelKind::Pulley {
            default_radius: 0.05,
        },
        "sag" => {
            use spyder_cables::Sag;
            spyder_core::CableModelKind::Sag(Sag::default())
        }
        other => return Err(format!("unknown cable model: {other}")),
    };
    Ok(())
}

/// Classify the current robot restraint.
pub fn classify_robot(robot: &Robot) -> Result<String, String> {
    robot
        .classify()
        .map(|c| c.as_str().to_string())
        .map_err(|e| e.to_string())
}
