//! Cable → device/axis routing for multi-board setups.

use serde::{Deserialize, Serialize};

use crate::{Result, RuntimeError};

/// One physical motor endpoint.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AxisEndpoint {
    /// Transport device: serial path or `host:port`.
    pub device: String,
    /// Baud (ignored for TCP).
    pub baud: u32,
    /// Axis index on that device (ODrive 0/1, or stepper board axis).
    pub axis: u8,
    /// Steps per revolution for this motor.
    pub steps_per_rev: f64,
}

/// Map from cable index → endpoint (length must equal cable count).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AxisMap {
    /// Ordered endpoints, one per cable.
    pub cables: Vec<AxisEndpoint>,
}

impl AxisMap {
    /// Validate non-empty and positive steps_per_rev.
    pub fn validate(&self) -> Result<()> {
        if self.cables.is_empty() {
            return Err(RuntimeError::Config("axis map empty".into()));
        }
        for (i, c) in self.cables.iter().enumerate() {
            if c.steps_per_rev <= 0.0 {
                return Err(RuntimeError::Config(format!(
                    "cable {i} steps_per_rev must be > 0"
                )));
            }
            if c.device.is_empty() {
                return Err(RuntimeError::Config(format!("cable {i} device empty")));
            }
        }
        Ok(())
    }

    /// Group cables by device string (preserves first-seen order of devices).
    pub fn devices(&self) -> Vec<String> {
        let mut out = Vec::new();
        for c in &self.cables {
            if !out.iter().any(|d| d == &c.device) {
                out.push(c.device.clone());
            }
        }
        out
    }

    /// Load from JSON.
    pub fn load_json(path: &std::path::Path) -> Result<Self> {
        let text =
            std::fs::read_to_string(path).map_err(|e| RuntimeError::Backend(e.to_string()))?;
        let map: Self =
            serde_json::from_str(&text).map_err(|e| RuntimeError::Config(e.to_string()))?;
        map.validate()?;
        Ok(map)
    }

    /// Save JSON.
    pub fn save_json(&self, path: &std::path::Path) -> Result<()> {
        self.validate()?;
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| RuntimeError::Config(e.to_string()))?;
        std::fs::write(path, text).map_err(|e| RuntimeError::Backend(e.to_string()))
    }

    /// Example 4-cable map on two ODrives (2 axes each) over two serial ports.
    pub fn example_dual_odrive() -> Self {
        Self {
            cables: vec![
                AxisEndpoint {
                    device: "/dev/ttyACM0".into(),
                    baud: 115200,
                    axis: 0,
                    steps_per_rev: 200.0,
                },
                AxisEndpoint {
                    device: "/dev/ttyACM0".into(),
                    baud: 115200,
                    axis: 1,
                    steps_per_rev: 200.0,
                },
                AxisEndpoint {
                    device: "/dev/ttyACM1".into(),
                    baud: 115200,
                    axis: 0,
                    steps_per_rev: 200.0,
                },
                AxisEndpoint {
                    device: "/dev/ttyACM1".into(),
                    baud: 115200,
                    axis: 1,
                    steps_per_rev: 200.0,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_devices() {
        let m = AxisMap::example_dual_odrive();
        m.validate().unwrap();
        assert_eq!(m.devices().len(), 2);
    }
}
