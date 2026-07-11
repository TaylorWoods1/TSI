//! Field calibration service wrappers.

use spyder_core::Vec3;
use spyder_runtime::{apply_anchor_override, Calibration};

use crate::dto::{
    CalibrationAnchorRequest, CalibrationCaptureRequest, CalibrationDto, CalibrationJsonResponse,
    CalibrationLoadRequest, VenueResponse,
};
use crate::state::{calibration_to_dto, classify_robot, venue_from_state, AppState};

/// Current calibration snapshot or defaults from venue.
pub async fn get_calibration(state: &AppState) -> CalibrationDto {
    let cal = state.calibration.lock().await;
    if let Some(ref c) = *cal {
        return calibration_to_dto(c);
    }
    let robot = state.robot.lock().await;
    let home = *state.home.lock().await;
    let drum = state
        .motor_axes
        .lock()
        .await
        .first()
        .map(|a| a.drum_radius_m)
        .unwrap_or(0.05);
    let spr = state
        .motor_axes
        .lock()
        .await
        .first()
        .map(|a| a.steps_per_rev)
        .unwrap_or(200.0);
    match Calibration::capture(&robot, home, drum, spr) {
        Ok(cal) => calibration_to_dto(&cal),
        Err(_) => CalibrationDto {
            home: [home.x, home.y, home.z],
            home_lengths_m: vec![],
            drum_radius_m: drum,
            steps_per_rev: spr,
            anchors_m: None,
            saved_at: "unset".into(),
        },
    }
}

/// Capture calibration at home.
pub async fn capture_calibration(
    state: &AppState,
    req: &CalibrationCaptureRequest,
) -> Result<CalibrationDto, String> {
    let robot = state.robot.lock().await;
    let home_vec = if let Some(h) = req.home {
        Vec3::new(h[0], h[1], h[2])
    } else {
        *state.home.lock().await
    };
    let cal = Calibration::capture(&robot, home_vec, req.drum_radius_m, req.steps_per_rev)
        .map_err(|e| e.to_string())?;
    drop(robot);
    *state.calibration.lock().await = Some(cal.clone());
    Ok(calibration_to_dto(&cal))
}

/// Override one measured anchor exit.
pub async fn set_calibration_anchor(
    state: &AppState,
    req: &CalibrationAnchorRequest,
) -> Result<CalibrationDto, String> {
    let mut cal = state
        .calibration
        .lock()
        .await
        .clone()
        .ok_or_else(|| "capture calibration first".to_string())?;
    let mut anchors = cal.anchors_m.clone().unwrap_or_default();
    if req.index >= anchors.len() {
        return Err("anchor index out of range".into());
    }
    anchors[req.index] = req.exit;
    cal.anchors_m = Some(anchors);
    *state.calibration.lock().await = Some(cal.clone());
    Ok(calibration_to_dto(&cal))
}

/// Apply calibration anchors + home to robot state.
pub async fn apply_calibration(state: &AppState) -> Result<VenueResponse, String> {
    let cal = state
        .calibration
        .lock()
        .await
        .clone()
        .ok_or_else(|| "no calibration to apply".to_string())?;
    {
        let mut robot = state.robot.lock().await;
        if let Some(anchors) = &cal.anchors_m {
            apply_anchor_override(&mut robot, anchors).map_err(|e| e.to_string())?;
        }
    }
    {
        let mut home = state.home.lock().await;
        *home = Vec3::new(cal.home[0], cal.home[1], cal.home[2]);
    }
    let venue = venue_from_state(state).await;
    let robot = state.robot.lock().await;
    let classify = classify_robot(&robot)?;
    Ok(VenueResponse { venue, classify })
}

/// Export calibration JSON.
pub async fn calibration_json(state: &AppState) -> Result<CalibrationJsonResponse, String> {
    let cal = state
        .calibration
        .lock()
        .await
        .clone()
        .ok_or_else(|| "no calibration captured".to_string())?;
    let json = serde_json::to_string_pretty(&cal).map_err(|e| e.to_string())?;
    Ok(CalibrationJsonResponse { json })
}

/// Load calibration from JSON text.
pub async fn load_calibration(
    state: &AppState,
    req: &CalibrationLoadRequest,
) -> Result<CalibrationDto, String> {
    let cal: Calibration =
        serde_json::from_str(&req.json).map_err(|e| format!("invalid calibration json: {e}"))?;
    *state.calibration.lock().await = Some(cal.clone());
    Ok(calibration_to_dto(&cal))
}

/// Export venue TOML merged from captured calibration (measured anchors + home).
pub async fn calibration_venue_toml(state: &AppState) -> Result<String, String> {
    let cal = state
        .calibration
        .lock()
        .await
        .clone()
        .ok_or_else(|| "no calibration captured".to_string())?;
    let robot = state.robot.lock().await;
    let point_mass = robot.point_mass;
    drop(robot);
    cal.to_venue_toml(point_mass).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capture_and_apply_round_trip() {
        let state = AppState::new_rect();
        let dto = capture_calibration(
            &state,
            &CalibrationCaptureRequest {
                home: None,
                drum_radius_m: 0.05,
                steps_per_rev: 200.0,
            },
        )
        .await
        .unwrap();
        assert_eq!(dto.home_lengths_m.len(), 4);
        let applied = apply_calibration(&state).await.unwrap();
        assert_eq!(applied.venue.anchors.len(), 4);
    }

    #[tokio::test]
    async fn venue_toml_export_requires_capture() {
        let state = AppState::new_rect();
        assert!(calibration_venue_toml(&state).await.is_err());
        capture_calibration(
            &state,
            &CalibrationCaptureRequest {
                home: None,
                drum_radius_m: 0.05,
                steps_per_rev: 200.0,
            },
        )
        .await
        .unwrap();
        let toml = calibration_venue_toml(&state).await.unwrap();
        assert!(toml.contains("[[anchors]]"));
        assert!(toml.contains("[home]"));
    }
}
