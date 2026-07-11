//! Per-cable motor mapping (drum radius, steps/rev).

use spyder_runtime::Axis;

use crate::dto::{MotorAxisDto, MotorsResponse, SetMotorsRequest};
use crate::state::AppState;

const DEFAULT_DRUM: f64 = 0.05;
const DEFAULT_SPR: f64 = 200.0;

/// Read motor axes, padding to cable count with defaults.
pub async fn get_motors(state: &AppState) -> MotorsResponse {
    let robot = state.robot.lock().await;
    let n = robot.anchors.len();
    let stored = state.motor_axes.lock().await.clone();
    MotorsResponse {
        axes: pad_motor_axes(&stored, n),
    }
}

/// Replace motor mapping for all cables.
pub async fn set_motors(
    state: &AppState,
    req: &SetMotorsRequest,
) -> Result<MotorsResponse, String> {
    let robot = state.robot.lock().await;
    let n = robot.anchors.len();
    if req.axes.len() != n {
        return Err(format!("expected {n} motor axes, got {}", req.axes.len()));
    }
    for (i, a) in req.axes.iter().enumerate() {
        if a.drum_radius_m <= 0.0 {
            return Err(format!("cable {i} drum_radius_m must be > 0"));
        }
        if a.steps_per_rev <= 0.0 {
            return Err(format!("cable {i} steps_per_rev must be > 0"));
        }
    }
    drop(robot);
    *state.motor_axes.lock().await = req.axes.clone();
    Ok(MotorsResponse {
        axes: req.axes.clone(),
    })
}

/// Pad or trim motor DTOs to match cable count.
pub fn pad_motor_axes(stored: &[MotorAxisDto], n: usize) -> Vec<MotorAxisDto> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(
            stored
                .get(i)
                .cloned()
                .unwrap_or(MotorAxisDto {
                    drum_radius_m: DEFAULT_DRUM,
                    steps_per_rev: DEFAULT_SPR,
                }),
        );
    }
    out
}

/// Build runtime axes from stored motor DTOs.
pub fn axes_from_state(motor_axes: &[MotorAxisDto], n: usize) -> Result<Vec<Axis>, String> {
    let dtos = pad_motor_axes(motor_axes, n);
    let mut axes = Vec::with_capacity(n);
    for dto in &dtos {
        axes.push(
            Axis::new(dto.drum_radius_m, dto.steps_per_rev, 1.0)
                .map_err(|e| e.to_string())?,
        );
    }
    Ok(axes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_defaults() {
        let axes = pad_motor_axes(&[], 4);
        assert_eq!(axes.len(), 4);
        assert!((axes[0].drum_radius_m - DEFAULT_DRUM).abs() < 1e-9);
    }
}
