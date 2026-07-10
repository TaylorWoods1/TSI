//! JSON request/response types for the GUI API.

use serde::{Deserialize, Serialize};
use spyder_core::Vec3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vec3Dto {
    /// X coordinate (meters).
    pub x: f64,
    /// Y coordinate (meters).
    pub y: f64,
    /// Z coordinate (meters).
    pub z: f64,
}

impl From<Vec3> for Vec3Dto {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3Dto> for Vec3 {
    fn from(v: Vec3Dto) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

impl From<[f64; 3]> for Vec3Dto {
    fn from(v: [f64; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VenueDto {
    /// Anchor exit positions.
    pub anchors: Vec<Vec3Dto>,
    /// Platform attachment points (body frame).
    pub attachments: Vec<Vec3Dto>,
    /// Point-mass vs rigid platform.
    pub point_mass: bool,
    /// Cable model: `ideal`, `pulley`, or `sag`.
    pub model: String,
    /// Default pulley radius when model is `pulley`.
    #[serde(default = "default_pulley_radius")]
    pub pulley_radius: f64,
    /// Sag mass per unit length (kg/m).
    #[serde(default = "default_sag_mu")]
    pub sag_mu: f64,
    /// Sag axial stiffness EA (N).
    #[serde(default = "default_sag_ea")]
    pub sag_ea: f64,
    /// Home pose.
    pub home: Vec3Dto,
}

fn default_pulley_radius() -> f64 {
    0.05
}
fn default_sag_mu() -> f64 {
    1.0
}
fn default_sag_ea() -> f64 {
    1.0e6
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VenueResponse {
    /// Current venue configuration.
    pub venue: VenueDto,
    /// Restraint classification (`IRPM` / `CRPM` / `RRPM`).
    pub classify: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Server is healthy.
    pub ok: bool,
    /// Crate version.
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Human-readable error message.
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadVenueRequest {
    /// Venue TOML text.
    pub toml: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FromPresetRequest {
    /// `rect` or `polygon`.
    pub kind: String,
    /// Rectangle width (meters).
    #[serde(default)]
    pub width: Option<f64>,
    /// Rectangle depth (meters).
    #[serde(default)]
    pub depth: Option<f64>,
    /// Anchor height (meters).
    #[serde(default)]
    pub height: Option<f64>,
    /// Polygon cable count.
    #[serde(default)]
    pub n: Option<usize>,
    /// Polygon circumradius (meters).
    #[serde(default)]
    pub radius: Option<f64>,
    /// Point-mass mode.
    #[serde(default = "default_true")]
    pub point_mass: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetAnchorsRequest {
    /// Anchor positions.
    pub anchors: Vec<Vec3Dto>,
    /// Optional attachment points.
    #[serde(default)]
    pub attachments: Option<Vec<Vec3Dto>>,
    /// Point-mass mode.
    #[serde(default = "default_true")]
    pub point_mass: bool,
    /// Cable model override.
    #[serde(default)]
    pub model: Option<String>,
    /// Pulley default radius (meters).
    #[serde(default)]
    pub pulley_radius: Option<f64>,
    /// Sag μ (kg/m).
    #[serde(default)]
    pub sag_mu: Option<f64>,
    /// Sag EA (N).
    #[serde(default)]
    pub sag_ea: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetCableModelRequest {
    /// `ideal`, `pulley`, or `sag`.
    pub model: String,
    /// Pulley default radius (meters).
    #[serde(default)]
    pub pulley_radius: Option<f64>,
    /// Sag μ (kg/m).
    #[serde(default)]
    pub sag_mu: Option<f64>,
    /// Sag EA (N).
    #[serde(default)]
    pub sag_ea: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TomlResponse {
    /// Serialized venue TOML.
    pub toml: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IkRequest {
    /// Target position `[x, y, z]`.
    pub xyz: [f64; 3],
    /// Cable model override.
    #[serde(default)]
    pub model: Option<String>,
    /// Cable model override.
    #[serde(default)]
    pub model: Option<String>,
    /// Gravity magnitude for wrench (Newtons); required for sag model.
    #[serde(default)]
    pub mg: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IkResponse {
    /// Cable lengths (meters).
    pub lengths: Vec<f64>,
    /// Per-cable tensions when wrench provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tensions: Option<Vec<f64>>,
    /// Unstrained lengths (sag model).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unstrained_lengths: Option<Vec<Option<f64>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FkRequest {
    /// Measured cable lengths.
    pub lengths: Vec<f64>,
    /// Seed position for numeric FK.
    pub seed: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FkResponse {
    /// Recovered position.
    pub xyz: [f64; 3],
    /// Orientation as rotation vector.
    pub orientation_rv: [f64; 3],
    /// Algorithm used.
    pub method: String,
    /// Residual norm.
    pub residual: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JacobianRequest {
    /// Pose position.
    pub xyz: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JacobianResponse {
    /// Jacobian rows (one per cable).
    pub rows: Vec<[f64; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeasibleRequest {
    /// Pose position.
    pub xyz: [f64; 3],
    /// Gravity magnitude (Newtons).
    #[serde(default = "default_mg")]
    pub mg: f64,
    /// Minimum tension (Newtons).
    #[serde(default = "default_f_min")]
    pub f_min: f64,
    /// Maximum tension (Newtons).
    #[serde(default = "default_f_max")]
    pub f_max: f64,
}

fn default_mg() -> f64 {
    9.81
}
fn default_f_min() -> f64 {
    0.5
}
fn default_f_max() -> f64 {
    500.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeasibleResponse {
    /// Whether the pose is wrench-feasible.
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceRequest {
    /// Box minimum corner.
    pub min: [f64; 3],
    /// Box maximum corner.
    pub max: [f64; 3],
    /// Samples along X.
    pub nx: usize,
    /// Samples along Y.
    pub ny: usize,
    /// Samples along Z.
    pub nz: usize,
    /// Gravity magnitude (Newtons).
    pub mg: f64,
    /// Minimum tension (Newtons).
    pub f_min: f64,
    /// Maximum tension (Newtons).
    pub f_max: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSampleDto {
    /// Sample X (meters).
    pub x: f64,
    /// Sample Y (meters).
    pub y: f64,
    /// Sample Z (meters).
    pub z: f64,
    /// Whether the pose is wrench-feasible.
    pub feasible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceResponse {
    /// Fraction of samples that are feasible.
    pub fraction: f64,
    /// Per-grid-point results.
    pub samples: Vec<WorkspaceSampleDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrajLineRequest {
    /// Start position.
    pub start: [f64; 3],
    /// End position.
    pub end: [f64; 3],
    /// Number of segments.
    pub segments: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrajLineResponse {
    /// Cartesian waypoints.
    pub waypoints: Vec<[f64; 3]>,
    /// IK lengths per waypoint.
    pub lengths: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneSnapshotRequest {
    /// Pose position.
    pub xyz: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneSnapshotResponse {
    /// Anchor world positions.
    pub anchors: Vec<[f64; 3]>,
    /// Dolly position.
    pub dolly: [f64; 3],
    /// Attachment world points.
    pub attachments: Vec<[f64; 3]>,
    /// Cable lengths at the pose.
    pub lengths: Vec<f64>,
    /// Model-aware cable polylines.
    pub cable_paths: Vec<Vec<[f64; 3]>>,
    /// Unit pull directions at attachments.
    pub unit_pulls: Vec<[f64; 3]>,
    /// Active cable model.
    pub model: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OkResponse {
    /// Operation succeeded.
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    /// `mock`, `stepper`, `odrive`, or `multiboard` (GUI MVP: mock only).
    pub backend: String,
    /// Serial path or `host:port` (hardware backends).
    #[serde(default)]
    pub device: Option<String>,
    /// Serial baud rate.
    #[serde(default)]
    pub baud: Option<u32>,
    /// Multi-board axis map JSON.
    #[serde(default)]
    pub axis_map: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectResponse {
    /// Connection succeeded.
    pub ok: bool,
    /// Number of motor axes.
    pub axes: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayLineRequest {
    /// Line start `[x, y, z]`.
    pub start: [f64; 3],
    /// Line end `[x, y, z]`.
    pub end: [f64; 3],
    /// Number of Cartesian segments.
    pub segments: usize,
    /// Enable closed-loop correction after each segment.
    pub closed_loop: bool,
    /// Sleep for segment duration (wall-clock playback).
    pub realtime: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayLineResponse {
    /// Final motor step counts per axis.
    pub final_steps: Vec<i64>,
    /// FK pose from feedback steps, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_pose: Option<[f64; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunStatusResponse {
    /// Whether a run session is active.
    pub connected: bool,
    /// Backend name when connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// E-stop latched.
    pub estopped: bool,
    /// Latest step positions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<i64>>,
    /// Latest feedback pose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pose: Option<[f64; 3]>,
}
