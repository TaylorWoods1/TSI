//! HTTP route handlers.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use crate::design::{from_preset, load_venue, set_anchors, set_cable_model, venue_toml};
use crate::dto::*;
use crate::run_svc::RunSession;
use crate::sim_svc::{feasible, fk, ik, jacobian, scene_snapshot, traj_line, workspace};
use crate::state::AppState;
use spyder_runtime::MotorBackend;

async fn run_connect(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, ApiError> {
    if req.backend != "mock" {
        return Err(ApiError(format!(
            "only mock backend supported in GUI MVP (got {})",
            req.backend
        )));
    }
    let robot = state.robot.lock().await;
    let home = *state.home.lock().await;
    let n = robot.anchors.len();
    let session = RunSession::connect_mock(&robot, home)?;
    drop(robot);
    *state.run_session.lock().await = Some(session);
    Ok(Json(ConnectResponse { ok: true, axes: n }))
}

async fn run_disconnect(State(state): State<Arc<AppState>>) -> Json<OkResponse> {
    *state.run_session.lock().await = None;
    Json(OkResponse { ok: true })
}

async fn run_home(State(state): State<Arc<AppState>>) -> Result<Json<OkResponse>, ApiError> {
    let mut session = state.run_session.lock().await;
    let session = session.as_mut().ok_or_else(|| ApiError("not connected".into()))?;
    if session.estopped {
        return Err(ApiError("e-stop latched".into()));
    }
    session.mock.home_hardware().map_err(|e| ApiError(e.to_string()))?;
    session.last_steps = session.mock.positions().to_vec();
    Ok(Json(OkResponse { ok: true }))
}

async fn run_play_line(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PlayLineRequest>,
) -> Result<Json<PlayLineResponse>, ApiError> {
    let robot = state.robot.lock().await;
    let mut session = state.run_session.lock().await;
    let session = session.as_mut().ok_or_else(|| ApiError("not connected".into()))?;
    Ok(Json(session.play_line(&robot, &req)?))
}

async fn run_estop(State(state): State<Arc<AppState>>) -> Result<Json<OkResponse>, ApiError> {
    let mut session = state.run_session.lock().await;
    let session = session.as_mut().ok_or_else(|| ApiError("not connected".into()))?;
    session.estop()?;
    Ok(Json(OkResponse { ok: true }))
}

async fn run_clear_estop(State(state): State<Arc<AppState>>) -> Result<Json<OkResponse>, ApiError> {
    let mut session = state.run_session.lock().await;
    let session = session.as_mut().ok_or_else(|| ApiError("not connected".into()))?;
    session.clear_estop();
    Ok(Json(OkResponse { ok: true }))
}

async fn run_status(State(state): State<Arc<AppState>>) -> Json<RunStatusResponse> {
    let robot = state.robot.lock().await;
    let session = state.run_session.lock().await;
    if let Some(ref s) = *session {
        Json(s.status(&robot))
    } else {
        Json(RunStatusResponse {
            connected: false,
            backend: None,
            estopped: false,
            steps: None,
            pose: None,
        })
    }
}

/// Build the API router.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/venue/load", post(venue_load))
        .route("/venue/from_preset", post(venue_from_preset))
        .route("/venue/set_anchors", post(venue_set_anchors))
        .route("/venue/set_model", post(venue_set_model))
        .route("/venue/toml", get(venue_toml_get))
        .route("/ik", post(ik_route))
        .route("/fk", post(fk_route))
        .route("/jacobian", post(jacobian_route))
        .route("/feasible", post(feasible_route))
        .route("/workspace", post(workspace_route))
        .route("/traj/line", post(traj_line_route))
        .route("/scene/snapshot", post(scene_snapshot_route))
        .route("/run/connect", post(run_connect))
        .route("/run/disconnect", post(run_disconnect))
        .route("/run/home", post(run_home))
        .route("/run/play_line", post(run_play_line))
        .route("/run/estop", post(run_estop))
        .route("/run/clear_estop", post(run_clear_estop))
        .route("/run/status", get(run_status))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: crate::version().into(),
    })
}

async fn venue_load(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoadVenueRequest>,
) -> Result<Json<VenueResponse>, ApiError> {
    Ok(Json(load_venue(&state, &req.toml).await?))
}

async fn venue_from_preset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FromPresetRequest>,
) -> Result<Json<VenueResponse>, ApiError> {
    Ok(Json(from_preset(&state, &req).await?))
}

async fn venue_set_anchors(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetAnchorsRequest>,
) -> Result<Json<VenueResponse>, ApiError> {
    Ok(Json(set_anchors(&state, &req).await?))
}

async fn venue_set_model(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetCableModelRequest>,
) -> Result<Json<VenueResponse>, ApiError> {
    Ok(Json(set_cable_model(&state, &req).await?))
}

async fn venue_toml_get(State(state): State<Arc<AppState>>) -> Result<Json<TomlResponse>, ApiError> {
    Ok(Json(TomlResponse {
        toml: venue_toml(&state).await?,
    }))
}

async fn ik_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IkRequest>,
) -> Result<Json<IkResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(ik(&robot, &req)?))
}

async fn fk_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FkRequest>,
) -> Result<Json<FkResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(fk(&robot, &req)?))
}

async fn jacobian_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<JacobianRequest>,
) -> Result<Json<JacobianResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(jacobian(&robot, &req)?))
}

async fn feasible_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FeasibleRequest>,
) -> Result<Json<FeasibleResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(feasible(&robot, &req)?))
}

async fn workspace_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WorkspaceRequest>,
) -> Result<Json<WorkspaceResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(workspace(&robot, &req)))
}

async fn traj_line_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TrajLineRequest>,
) -> Result<Json<TrajLineResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(traj_line(&robot, &req)?))
}

async fn scene_snapshot_route(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SceneSnapshotRequest>,
) -> Result<Json<SceneSnapshotResponse>, ApiError> {
    let robot = state.robot.lock().await;
    Ok(Json(scene_snapshot(&robot, &req)?))
}

/// API error type mapping to HTTP 4xx + JSON body.
pub struct ApiError(String);

impl From<String> for ApiError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody { error: self.0 });
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn body_json<T: serde::de::DeserializeOwned>(resp: axum::response::Response) -> T {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_ok() {
        let app = router(AppState::new_rect());
        let resp = app
            .oneshot(
                http::Request::builder()
                    .uri("/health")
                    .body(http_body_util::Empty::<Bytes>::new())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: HealthResponse = body_json(resp).await;
        assert!(body.ok);
        assert_eq!(body.version, crate::version());
    }

    #[tokio::test]
    async fn venue_from_preset_rect() {
        let app = router(AppState::new_rect());
        let body = serde_json::json!({
            "kind": "rect",
            "width": 10.0,
            "depth": 6.0,
            "height": 8.0,
            "point_mass": true
        });
        let resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/venue/from_preset")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v: VenueResponse = body_json(resp).await;
        assert_eq!(v.venue.anchors.len(), 4);
        assert_eq!(v.classify, "RRPM");
    }

    #[tokio::test]
    async fn ik_returns_lengths() {
        let app = router(AppState::new_rect());
        let body = serde_json::json!({ "xyz": [0.0, 0.0, 2.0] });
        let resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/ik")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v: IkResponse = body_json(resp).await;
        assert_eq!(v.lengths.len(), 4);
    }

    #[tokio::test]
    async fn workspace_has_feasible_points() {
        let app = router(AppState::new_rect());
        let body = serde_json::json!({
            "min": [-2.0, -2.0, 0.5],
            "max": [2.0, 2.0, 4.0],
            "nx": 5,
            "ny": 5,
            "nz": 4,
            "mg": 9.81,
            "f_min": 0.5,
            "f_max": 500.0
        });
        let resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/workspace")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v: WorkspaceResponse = body_json(resp).await;
        assert!(v.fraction > 0.0, "expected feasible fraction > 0");
    }

    #[tokio::test]
    async fn mock_run_play_nonzero_steps() {
        let state = AppState::new_rect();
        let app = router(state.clone());
        let connect = serde_json::json!({ "backend": "mock" });
        let resp = app
            .clone()
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/run/connect")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(connect.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        let play = serde_json::json!({
            "start": [0.0, 0.0, 2.0],
            "end": [0.5, 0.0, 2.0],
            "segments": 5,
            "closed_loop": false,
            "realtime": false
        });
        let resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/run/play_line")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(play.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v: PlayLineResponse = body_json(resp).await;
        assert!(v.final_steps.iter().any(|s| *s != 0));
    }

    #[tokio::test]
    async fn fk_round_trip_after_ik() {
        let app = router(AppState::new_rect());
        let ik_body = serde_json::json!({ "xyz": [0.0, 0.0, 2.0] });
        let ik_resp = app
            .clone()
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/ik")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(ik_body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        let ik: IkResponse = body_json(ik_resp).await;

        let fk_body = serde_json::json!({
            "lengths": ik.lengths,
            "seed": [0.0, 0.0, 2.0]
        });
        let fk_resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/fk")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(fk_body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(fk_resp.status(), 200);
        let fk: FkResponse = body_json(fk_resp).await;
        assert!((fk.xyz[2] - 2.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn invalid_toml_returns_400() {
        let app = router(AppState::new_rect());
        let body = serde_json::json!({ "toml": "preset = \"rect\"\nwidth = oops" });
        let resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/venue/load")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn run_status_disconnected_by_default() {
        let app = router(AppState::new_rect());
        let resp = app
            .oneshot(
                http::Request::builder()
                    .uri("/run/status")
                    .body(http_body_util::Empty::<Bytes>::new())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let s: RunStatusResponse = body_json(resp).await;
        assert!(!s.connected);
    }

    #[tokio::test]
    async fn traj_line_returns_waypoints() {
        let app = router(AppState::new_rect());
        let body = serde_json::json!({
            "start": [0.0, 0.0, 2.0],
            "end": [0.5, 0.0, 2.0],
            "segments": 4
        });
        let resp = app
            .oneshot(
                http::Request::builder()
                    .method("POST")
                    .uri("/traj/line")
                    .header("content-type", "application/json")
                    .body(http_body_util::Full::new(Bytes::from(body.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let v: TrajLineResponse = body_json(resp).await;
        assert_eq!(v.waypoints.len(), 5);
        assert_eq!(v.lengths.len(), 5);
    }
}
