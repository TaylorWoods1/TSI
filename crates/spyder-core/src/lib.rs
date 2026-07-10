//! Spyder core: frames, poses, anchors, presets, IK, and FK orchestration.
//!
//! World frame is right-handed, **Z-up**, units in meters.

#![deny(missing_docs)]

pub mod anchor;
pub mod cable_eval;
pub mod cable_path;
pub mod error;
pub mod fk;
pub mod fk_analytic;
pub mod ik;
pub mod jacobian;
pub mod pose;
pub mod preset;
pub mod robot;
pub mod types;

pub use anchor::{Anchor, PlatformAttachment};
pub use cable_eval::{cable_geometry_at, default_pulley_radius, predicted_lengths, unit_pulls_at_pose};
pub use cable_path::cable_paths_at_pose;
pub use error::{Result, SpyderError};
pub use fk::{
    fk_platform_numeric, fk_point_mass_from_anchors, fk_point_mass_numeric, FkMethod, FkOptions,
    FkResult,
};
pub use fk_analytic::{fk_analytic_3, fk_analytic_rect4, is_axis_aligned_rect4};
pub use ik::{
    apply_ik_options, ideal_ik_point_mass, ik_ideal, ik_with_model, IkOptions, IkResult,
};
pub use jacobian::{
    length_jacobian, length_jacobian_platform_6, length_jacobian_platform_6_with_pulls,
    length_jacobian_point_mass,
};
pub use pose::Pose;
pub use preset::{rect, regular_polygon, triangle};
pub use robot::{CableModelKind, Preset, Robot};
pub use spyder_statics::{classify_restraint, RestraintClass};
pub use types::{Mat3, UnitQuat, Vec3};
