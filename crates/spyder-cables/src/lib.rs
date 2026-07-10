//! Cable length models for spyder CDPRs.

#![deny(missing_docs)]

pub mod ideal;
pub mod model;

pub use ideal::Ideal;
pub use model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};
