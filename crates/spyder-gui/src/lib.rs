//! Spyder local GUI HTTP service.

pub mod api;
pub mod design;
pub mod dto;
pub mod run_svc;
pub mod sim_svc;
pub mod state;
pub mod toml_venue;

pub use state::AppState;

/// Crate version string.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
