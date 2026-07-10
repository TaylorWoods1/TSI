//! Base exit points and platform attachment points.

use crate::types::Vec3;

/// Cable exit / pulley location in the world frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    /// Point where the cable leaves the base structure into the workspace.
    pub exit: Vec3,
    /// Optional unit swivel axis for a pulley at the exit (world frame).
    pub pulley_axis: Option<Vec3>,
    /// Pulley radius in meters (0 = treat as point exit).
    pub pulley_radius: f64,
}

impl Anchor {
    /// Point exit with no pulley compensation.
    pub fn point(exit: Vec3) -> Self {
        Self {
            exit,
            pulley_axis: None,
            pulley_radius: 0.0,
        }
    }

    /// Exit with a vertical (Z) swivel pulley.
    pub fn with_z_pulley(exit: Vec3, radius: f64) -> Self {
        Self {
            exit,
            pulley_axis: Some(Vec3::z()),
            pulley_radius: radius,
        }
    }
}

/// Cable attachment on the moving platform, in the body frame.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformAttachment {
    /// Attachment point relative to the platform origin.
    pub body_point: Vec3,
}

impl PlatformAttachment {
    /// Coincident / point-mass attachment at the body origin.
    pub fn origin() -> Self {
        Self {
            body_point: Vec3::zeros(),
        }
    }

    /// Offset attachment in the body frame.
    pub fn at(body_point: Vec3) -> Self {
        Self { body_point }
    }
}
