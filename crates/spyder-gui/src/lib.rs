//! Spyder local GUI HTTP service.
//!
//! Serves a JSON API on port 7700 wrapping `spyder-core`, `spyder-sim`, and
//! `spyder-runtime`. Optionally serves the built React SPA from `web/dist`.
//!
//! # Modules
//!
//! - [`api`] — Axum route handlers
//! - [`dto`] — Serde request/response types
//! - [`state`] — `AppState` (robot + home + run session)
//! - [`design`] — Venue mutation helpers
//! - [`sim_svc`] — IK/FK/workspace/trajectory/scene services
//! - [`run_svc`] — Mock motor playback session
//! - [`toml_venue`] — Venue TOML parse/emit

/// HTTP route handlers and router.
pub mod api;
/// Venue mutation helpers (presets, anchors, load/save).
pub mod design;
/// Field calibration wrappers.
pub mod cal_svc;
/// JSON request/response types for the REST API.
pub mod dto;
/// Mock motor run session and playback.
pub mod run_svc;
/// Simulation helpers (workspace, trajectory, scene).
pub mod sim_svc;
/// Shared server state (`Robot`, home pose, run session).
pub mod state;
/// Venue TOML parse and emit.
pub mod toml_venue;

pub use state::AppState;

/// Crate version string.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
