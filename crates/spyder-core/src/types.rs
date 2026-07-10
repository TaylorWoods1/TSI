//! Common numeric aliases (meters, Z-up world frame).

/// 3D vector in meters.
pub type Vec3 = nalgebra::Vector3<f64>;
/// 3×3 matrix.
pub type Mat3 = nalgebra::Matrix3<f64>;
/// Unit quaternion for orientation.
pub type UnitQuat = nalgebra::UnitQuaternion<f64>;
