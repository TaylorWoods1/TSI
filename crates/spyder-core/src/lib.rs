//! Spyder core: frames, poses, anchors, presets, IK, and FK orchestration.
//!
//! World frame is right-handed, **Z-up**, units in meters.

#![deny(missing_docs)]

pub mod anchor;
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
pub use error::{Result, SpyderError};
pub use fk::{fk_point_mass_from_anchors, fk_point_mass_numeric, FkMethod, FkResult};
pub use fk_analytic::{fk_analytic_3, fk_analytic_rect4, is_axis_aligned_rect4};
pub use ik::{
    apply_ik_options, ideal_ik_point_mass, ik_ideal, ik_with_model, IkOptions, IkResult,
};
pub use jacobian::{length_jacobian, length_jacobian_point_mass};
pub use pose::Pose;
pub use preset::{rect, regular_polygon, triangle};
pub use robot::{CableModelKind, Preset, Robot};
pub use spyder_statics::{classify_restraint, RestraintClass};
pub use types::{Mat3, UnitQuat, Vec3};
