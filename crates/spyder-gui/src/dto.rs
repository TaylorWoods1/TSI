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
    /// Home pose.
    pub home: Vec3Dto,
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
    /// Gravity magnitude for wrench (Newtons); enables tensions.
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
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub feasible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceResponse {
    pub fraction: f64,
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
    pub anchors: Vec<[f64; 3]>,
    pub dolly: [f64; 3],
    pub attachments: Vec<[f64; 3]>,
    pub lengths: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OkResponse {
    pub ok: bool,
}
