//! Actuation: winch geometry and motor command mapping.

#![deny(missing_docs)]

pub mod mapping;
pub mod motor;
pub mod winch;

pub use mapping::{length_delta_to_command, synchronized_step_delays, MotorCommand};
pub use motor::{Motor, MotorError};
pub use winch::{Winch, WinchError};
