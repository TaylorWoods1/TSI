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

/// Rich anchor descriptor (exit + optional pulley geometry).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnchorDto {
    /// Exit X (meters).
    pub x: f64,
    /// Exit Y (meters).
    pub y: f64,
    /// Exit Z (meters).
    pub z: f64,
    /// Pulley swivel axis (world frame); default Z-up when pulley model active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pulley_axis: Option<Vec3Dto>,
    /// Per-anchor pulley radius override (0 = use venue default).
    #[serde(default)]
    pub pulley_radius: f64,
    /// Unit direction cable leaves rim toward winch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pulley_winch_exit: Option<Vec3Dto>,
    /// Constant rim-to-encoder runout (meters).
    #[serde(default)]
    pub pulley_runout_m: f64,
}

impl AnchorDto {
    /// Exit as core vector.
    pub fn exit(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VenueDto {
    /// Anchor exit positions and pulley fields.
    pub anchors: Vec<AnchorDto>,
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
pub struct SetHomeRequest {
    /// Home pose `[x, y, z]`.
    pub home: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetAnchorsRequest {
    /// Anchor positions (legacy flat `{x,y,z}` also accepted via untagged).
    #[serde(deserialize_with = "deserialize_anchors")]
    pub anchors: Vec<AnchorDto>,
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
    /// Gravity magnitude for wrench (Newtons); required for sag model.
    #[serde(default)]
    pub mg: Option<f64>,
    /// Reference cable lengths for motor command deltas (defaults to venue home IK).
    #[serde(default)]
    pub reference_lengths: Option<Vec<f64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorCommandDto {
    /// Winch rotation (radians).
    pub winch_radians: f64,
    /// Motor steps (rounded).
    pub steps: i64,
    /// Exact steps before rounding.
    pub steps_exact: f64,
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
    /// Motor commands vs reference lengths when motor mapping is configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motor_commands: Option<Vec<MotorCommandDto>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FkRequest {
    /// Measured cable lengths.
    pub lengths: Vec<f64>,
    /// Seed position for numeric FK.
    pub seed: [f64; 3],
    /// Seed orientation as rotation vector.
    #[serde(default)]
    pub orientation_rv: Option<[f64; 3]>,
    /// Per-cable tensions for sag FK.
    #[serde(default)]
    pub tensions: Option<Vec<f64>>,
    /// Allow underconstrained platform FK.
    #[serde(default)]
    pub allow_underconstrained: bool,
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
    /// Orientation rotation vector (platform mode).
    #[serde(default)]
    pub orientation_rv: Option<[f64; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JacobianResponse {
    /// Jacobian rows (one per cable); 3 or 6 columns.
    pub rows: Vec<Vec<f64>>,
    /// Column count (3 point-mass, 6 platform).
    pub cols: usize,
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
pub struct TrajWaypointsRequest {
    /// Cartesian waypoints.
    pub waypoints: Vec<[f64; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrajWaypointsResponse {
    /// Echo waypoints.
    pub waypoints: Vec<[f64; 3]>,
    /// IK lengths per waypoint.
    pub lengths: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneSnapshotRequest {
    /// Pose position.
    pub xyz: [f64; 3],
    /// Orientation rotation vector (platform mode).
    #[serde(default)]
    pub orientation_rv: Option<[f64; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneExportRequest {
    /// Pose position.
    pub xyz: [f64; 3],
    /// `html` or `html_anim`.
    #[serde(default = "default_html_format")]
    pub format: String,
    /// Optional waypoint list for animation export.
    #[serde(default)]
    pub waypoints: Option<Vec<[f64; 3]>>,
    /// Orientation rotation vector.
    #[serde(default)]
    pub orientation_rv: Option<[f64; 3]>,
}

fn default_html_format() -> String {
    "html".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneExportResponse {
    /// Self-contained Plotly HTML.
    pub html: String,
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
    /// `mock`, `stepper`, `odrive`, or `multiboard`.
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
    /// Multiboard dry-run: mock fan-out instead of opening serial devices.
    #[serde(default)]
    pub mock: bool,
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
pub struct PlayWaypointsRequest {
    /// Cartesian waypoints `[x, y, z]`.
    pub waypoints: Vec<[f64; 3]>,
    /// Total trajectory duration (seconds).
    pub duration_s: f64,
    /// Enable closed-loop correction after each segment.
    pub closed_loop: bool,
    /// Sleep for segment duration (wall-clock playback).
    pub realtime: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayWaypointsResponse {
    /// Final motor step counts per axis.
    pub final_steps: Vec<i64>,
    /// FK pose from feedback steps, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_pose: Option<[f64; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafetyLimitsDto {
    /// Workspace minimum corner.
    pub min: [f64; 3],
    /// Workspace maximum corner.
    pub max: [f64; 3],
    /// Max Cartesian speed (m/s).
    pub max_speed_mps: f64,
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
    /// Active safety limits summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<SafetyLimitsDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationDto {
    /// Home pose.
    pub home: [f64; 3],
    /// Ideal lengths at home.
    pub home_lengths_m: Vec<f64>,
    /// Drum radius (m).
    pub drum_radius_m: f64,
    /// Steps per revolution.
    pub steps_per_rev: f64,
    /// Measured anchor overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchors_m: Option<Vec<[f64; 3]>>,
    /// Saved timestamp.
    pub saved_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationCaptureRequest {
    /// Override home pose (defaults to venue home).
    #[serde(default)]
    pub home: Option<[f64; 3]>,
    /// Drum radius (m).
    pub drum_radius_m: f64,
    /// Steps per revolution.
    pub steps_per_rev: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationAnchorRequest {
    /// Anchor index.
    pub index: usize,
    /// Measured exit `[x, y, z]`.
    pub exit: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationLoadRequest {
    /// Calibration JSON text.
    pub json: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationJsonResponse {
    /// Pretty JSON.
    pub json: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorAxisDto {
    /// Drum radius (m).
    pub drum_radius_m: f64,
    /// Steps per revolution.
    pub steps_per_rev: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorsResponse {
    /// Per-cable motor mapping.
    pub axes: Vec<MotorAxisDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetMotorsRequest {
    /// One entry per cable.
    pub axes: Vec<MotorAxisDto>,
}

fn deserialize_anchors<'de, D>(deserializer: D) -> Result<Vec<AnchorDto>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Vec<AnchorDto>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("anchor array")
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut out = Vec::new();
            while let Some(v) = seq.next_element::<serde_json::Value>()? {
                out.push(parse_anchor_value(v).map_err(de::Error::custom)?);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_seq(V)
}

fn parse_anchor_value(v: serde_json::Value) -> Result<AnchorDto, String> {
    if let Some(exit) = v.get("exit") {
        let x = exit.get("x").and_then(|x| x.as_f64()).ok_or("missing x")?;
        let y = exit.get("y").and_then(|x| x.as_f64()).ok_or("missing y")?;
        let z = exit.get("z").and_then(|x| x.as_f64()).ok_or("missing z")?;
        return Ok(AnchorDto {
            x,
            y,
            z,
            pulley_axis: v.get("pulley_axis").and_then(parse_vec3),
            pulley_radius: v.get("pulley_radius").and_then(|x| x.as_f64()).unwrap_or(0.0),
            pulley_winch_exit: v.get("pulley_winch_exit").and_then(parse_vec3),
            pulley_runout_m: v.get("pulley_runout_m").and_then(|x| x.as_f64()).unwrap_or(0.0),
        });
    }
    let x = v.get("x").and_then(|x| x.as_f64()).ok_or("missing x")?;
    let y = v.get("y").and_then(|x| x.as_f64()).ok_or("missing y")?;
    let z = v.get("z").and_then(|x| x.as_f64()).ok_or("missing z")?;
    Ok(AnchorDto {
        x,
        y,
        z,
        pulley_axis: v.get("pulley_axis").and_then(parse_vec3),
        pulley_radius: v.get("pulley_radius").and_then(|x| x.as_f64()).unwrap_or(0.0),
        pulley_winch_exit: v.get("pulley_winch_exit").and_then(parse_vec3),
        pulley_runout_m: v.get("pulley_runout_m").and_then(|x| x.as_f64()).unwrap_or(0.0),
    })
}

fn parse_vec3(v: &serde_json::Value) -> Option<Vec3Dto> {
    Some(Vec3Dto {
        x: v.get("x")?.as_f64()?,
        y: v.get("y")?.as_f64()?,
        z: v.get("z")?.as_f64()?,
    })
}
