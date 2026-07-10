//! Cable length models for spyder CDPRs.

#![deny(missing_docs)]

pub mod ideal;
pub mod model;
pub mod pulley;
pub mod sag;

pub use ideal::Ideal;
pub use model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};
pub use pulley::Pulley;
pub use sag::Sag;
