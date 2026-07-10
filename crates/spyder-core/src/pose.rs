//! End-effector pose: position + orientation.

use crate::types::{UnitQuat, Vec3};

/// Rigid pose of the platform / point-mass in the world frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Pose {
    /// Position of the body origin in world coordinates (meters).
    pub position: Vec3,
    /// Orientation of the body frame relative to world.
    pub orientation: UnitQuat,
}

impl Pose {
    /// Identity pose at the origin with no rotation.
    pub fn identity() -> Self {
        Self {
            position: Vec3::zeros(),
            orientation: UnitQuat::identity(),
        }
    }

    /// Translation-only pose (identity orientation).
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            orientation: UnitQuat::identity(),
        }
    }

    /// Full pose from position and orientation.
    pub fn new(position: Vec3, orientation: UnitQuat) -> Self {
        Self {
            position,
            orientation,
        }
    }

    /// Transform a body-frame point into world coordinates: `p + R * b`.
    pub fn transform_point(&self, body_point: &Vec3) -> Vec3 {
        self.position + self.orientation * body_point
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn identity_pose_leaves_point_unchanged() {
        let pose = Pose::identity();
        let p = Vec3::new(1.0, 2.0, 3.0);
        let out = pose.transform_point(&p);
        assert_relative_eq!(out.x, 1.0);
        assert_relative_eq!(out.y, 2.0);
        assert_relative_eq!(out.z, 3.0);
    }

    #[test]
    fn translation_only_pose() {
        let pose = Pose::from_position(Vec3::new(1.0, 0.0, 0.0));
        let out = pose.transform_point(&Vec3::new(0.0, 2.0, 0.0));
        assert_relative_eq!(out.x, 1.0);
        assert_relative_eq!(out.y, 2.0);
        assert_relative_eq!(out.z, 0.0);
    }
}
