//! Base exit points and platform attachment points.

use crate::types::Vec3;

/// Cable exit / pulley location in the world frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    /// Point where the cable leaves the base structure into the workspace (pulley center).
    pub exit: Vec3,
    /// Optional unit swivel axis for a pulley at the exit (world frame).
    pub pulley_axis: Option<Vec3>,
    /// Pulley radius in meters (0 = treat as point exit).
    pub pulley_radius: f64,
    /// Unit direction the cable leaves the pulley rim toward the winch (⊥ axis when set).
    pub pulley_winch_exit: Option<Vec3>,
    /// Constant rim-to-encoder path length along the winch run (meters).
    pub pulley_runout_m: f64,
}

impl Anchor {
    /// Point exit with no pulley compensation.
    pub fn point(exit: Vec3) -> Self {
        Self {
            exit,
            pulley_axis: None,
            pulley_radius: 0.0,
            pulley_winch_exit: None,
            pulley_runout_m: 0.0,
        }
    }

    /// Exit with a vertical (Z) swivel pulley.
    pub fn with_z_pulley(exit: Vec3, radius: f64) -> Self {
        Self {
            exit,
            pulley_axis: Some(Vec3::z()),
            pulley_radius: radius,
            pulley_winch_exit: None,
            pulley_runout_m: 0.0,
        }
    }

    /// Exit with pulley axis, radius, and optional winch exit azimuth.
    pub fn with_pulley(exit: Vec3, axis: Vec3, radius: f64, winch_exit: Option<Vec3>) -> Self {
        Self {
            exit,
            pulley_axis: Some(axis),
            pulley_radius: radius,
            pulley_winch_exit: winch_exit,
            pulley_runout_m: 0.0,
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
