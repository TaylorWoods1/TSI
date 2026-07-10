//! Spyder CLI library surface.
//!
//! The binary (`spyder-cli`) implements IK, FK, workspace, scene, calibration,
//! and playback subcommands. This crate exposes the shared venue TOML parser so
//! tests and other crates can load [`spyder_core::Robot`] configs without
//! shelling out to the CLI.
//!
//! # Modules
//!
//! - [`toml`] — venue TOML → [`spyder_core::Robot`]

pub mod toml;

pub use toml::robot_from_toml;
