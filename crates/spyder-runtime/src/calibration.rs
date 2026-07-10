//! Homing / zero-length calibration helpers.

use serde::{Deserialize, Serialize};
use spyder_core::{Pose, Robot, Vec3};

use crate::{Result, RuntimeError};

/// Measured / configured calibration snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calibration {
    /// World pose considered "home" (meters).
    pub home: [f64; 3],
    /// Ideal IK cable lengths at home (meters) — geometric zero reference.
    pub home_lengths_m: Vec<f64>,
    /// Winch drum radius used for step mapping (meters).
    pub drum_radius_m: f64,
    /// Motor steps per winch revolution (after gearing).
    pub steps_per_rev: f64,
    /// Optional measured anchor exits overriding the robot preset.
    pub anchors_m: Option<Vec<[f64; 3]>>,
    /// ISO-ish timestamp string (informational).
    pub saved_at: String,
}

impl Calibration {
    /// Capture calibration from a robot at `home` with actuation params.
    pub fn capture(
        robot: &Robot,
        home: Vec3,
        drum_radius_m: f64,
        steps_per_rev: f64,
    ) -> Result<Self> {
        if drum_radius_m <= 0.0 || steps_per_rev <= 0.0 {
            return Err(RuntimeError::Config(
                "drum_radius and steps_per_rev must be > 0".into(),
            ));
        }
        let ik = robot.ik(&Pose::from_position(home))?;
        let anchors_m = Some(
            robot
                .anchors
                .iter()
                .map(|a| [a.exit.x, a.exit.y, a.exit.z])
                .collect(),
        );
        Ok(Self {
            home: [home.x, home.y, home.z],
            home_lengths_m: ik.lengths,
            drum_radius_m,
            steps_per_rev,
            anchors_m,
            saved_at: chrono_lite_now(),
        })
    }

    /// Home as Vec3.
    pub fn home_vec(&self) -> Vec3 {
        Vec3::new(self.home[0], self.home[1], self.home[2])
    }

    /// Write JSON calibration file.
    pub fn save_json(&self, path: &std::path::Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| RuntimeError::Config(e.to_string()))?;
        std::fs::write(path, text).map_err(|e| RuntimeError::Backend(e.to_string()))
    }

    /// Load JSON calibration file.
    pub fn load_json(path: &std::path::Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).map_err(|e| RuntimeError::Backend(e.to_string()))?;
        serde_json::from_str(&text).map_err(|e| RuntimeError::Config(e.to_string()))
    }
}

fn chrono_lite_now() -> String {
    // Avoid chrono dep: use system time since epoch.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

/// Apply measured anchors onto a robot (point-mass attachments preserved).
pub fn apply_anchor_override(robot: &mut Robot, anchors_m: &[[f64; 3]]) -> Result<()> {
    if anchors_m.len() != robot.anchors.len() {
        return Err(RuntimeError::Config(
            "anchor override count must match robot".into(),
        ));
    }
    for (dst, src) in robot.anchors.iter_mut().zip(anchors_m.iter()) {
        dst.exit = Vec3::new(src[0], src[1], src[2]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Preset;
    use std::path::PathBuf;

    #[test]
    fn capture_and_roundtrip_json() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let home = Vec3::new(0.0, 0.0, 1.2);
        let cal = Calibration::capture(&robot, home, 0.05, 200.0).unwrap();
        assert_eq!(cal.home_lengths_m.len(), 4);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-cal");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cal.json");
        cal.save_json(&path).unwrap();
        let loaded = Calibration::load_json(&path).unwrap();
        assert_eq!(loaded.home_lengths_m, cal.home_lengths_m);
    }
}
