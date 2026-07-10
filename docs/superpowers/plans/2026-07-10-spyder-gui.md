# Spyder GUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a local Design → Simulate → Run GUI over existing spyder crates (Axum API + Vite/React/Three.js SPA) without reimplementing IK.

**Architecture:** `crates/spyder-gui` Axum server holds `Robot` + optional run session; JSON API on `:7700`; `web/` SPA consumes API and renders R3F scene. Static `web/dist` embedded/served in release.

**Tech Stack:** Rust 2021 (axum 0.7, tokio, serde_json, tower-http), existing spyder-*, Vite 5, React 18, @react-three/fiber, three, Vitest, Playwright.

**Spec:** `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`

---

## File map

```
Cargo.toml                              # add spyder-gui member
crates/spyder-gui/Cargo.toml
crates/spyder-gui/src/main.rs
crates/spyder-gui/src/lib.rs            # AppState + router for tests
crates/spyder-gui/src/dto.rs
crates/spyder-gui/src/design.rs
crates/spyder-gui/src/sim_svc.rs
crates/spyder-gui/src/run_svc.rs
crates/spyder-gui/src/api.rs
crates/spyder-gui/src/toml_venue.rs     # parse/emit venue TOML (share logic w/ CLI later)
web/package.json
web/vite.config.ts
web/index.html
web/src/main.tsx
web/src/App.tsx
web/src/api/client.ts
web/src/scene/RobotScene.tsx
web/src/pages/DesignPage.tsx
web/src/pages/SimulatePage.tsx
web/src/pages/RunPage.tsx
web/src/styles.css
.github/workflows/ci.yml                # extend
README.md                               # GUI quickstart
```

---

## Milestone GUI-1 — API

### Task 1: Scaffold `spyder-gui` crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/spyder-gui/Cargo.toml`
- Create: `crates/spyder-gui/src/lib.rs`
- Create: `crates/spyder-gui/src/main.rs`

- [ ] **Step 1: Add workspace member**

In root `Cargo.toml` `members`, append `"crates/spyder-gui"`.

- [ ] **Step 2: Create package manifest**

```toml
# crates/spyder-gui/Cargo.toml
[package]
name = "spyder-gui"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Local Design/Simulate/Run GUI server for spyder"

[[bin]]
name = "spyder-gui"
path = "src/main.rs"

[dependencies]
spyder-core = { path = "../spyder-core" }
spyder-cables = { path = "../spyder-cables" }
spyder-sim = { path = "../spyder-sim" }
spyder-runtime = { path = "../spyder-runtime" }
nalgebra = { workspace = true }
serde = { workspace = true }
serde_json = "1"
thiserror = { workspace = true }
tokio = { version = "1", features = ["full"] }
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "fs"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
```

Pin axum/tower-http if Rust 1.83 requires older compatible versions — adjust to compile.

- [ ] **Step 3: Minimal lib + main**

```rust
// crates/spyder-gui/src/lib.rs
//! Spyder local GUI HTTP service.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

```rust
// crates/spyder-gui/src/main.rs
#[tokio::main]
async fn main() {
    println!("spyder-gui {}", spyder_gui::version());
}
```

- [ ] **Step 4: Verify**

Run: `cargo build -p spyder-gui`  
Expected: success

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/spyder-gui
git commit -m "chore: scaffold spyder-gui crate"
```

---

### Task 2: DTOs + AppState

**Files:**
- Create: `crates/spyder-gui/src/dto.rs`
- Create: `crates/spyder-gui/src/state.rs`
- Modify: `crates/spyder-gui/src/lib.rs`

- [ ] **Step 1: Define DTOs**

```rust
// dto.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vec3Dto {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<spyder_core::Vec3> for Vec3Dto {
    fn from(v: spyder_core::Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Vec3Dto> for spyder_core::Vec3 {
    fn from(v: Vec3Dto) -> Self {
        spyder_core::Vec3::new(v.x, v.y, v.z)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VenueDto {
    pub anchors: Vec<Vec3Dto>,
    pub attachments: Vec<Vec3Dto>,
    pub point_mass: bool,
    pub model: String, // "ideal" | "pulley" | "sag"
    pub home: Vec3Dto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}
```

- [ ] **Step 2: AppState**

```rust
// state.rs
use std::sync::Arc;
use tokio::sync::Mutex;
use spyder_core::{Preset, Robot, Vec3};

use crate::dto::VenueDto;

pub struct AppState {
    pub robot: Mutex<Robot>,
    pub home: Mutex<Vec3>,
    // run session filled in Task 7
}

impl AppState {
    pub fn new_rect() -> Arc<Self> {
        let robot = Robot::from_preset(Preset::Rect {
            width: 10.0,
            depth: 6.0,
            height: 8.0,
        })
        .expect("default robot");
        Arc::new(Self {
            robot: Mutex::new(robot),
            home: Mutex::new(Vec3::new(0.0, 0.0, 2.0)),
        })
    }
}

pub async fn venue_from_state(state: &AppState) -> VenueDto {
    let robot = state.robot.lock().await;
    let home = *state.home.lock().await;
    VenueDto {
        anchors: robot.anchors.iter().map(|a| a.exit.into()).collect(),
        attachments: robot.attachments.iter().map(|a| a.body_point.into()).collect(),
        point_mass: robot.point_mass,
        model: match &robot.cable_model {
            spyder_core::CableModelKind::Ideal => "ideal".into(),
            spyder_core::CableModelKind::Pulley { .. } => "pulley".into(),
            spyder_core::CableModelKind::Sag(_) => "sag".into(),
        },
        home: home.into(),
    }
}
```

- [ ] **Step 3: Export from lib**

```rust
pub mod dto;
pub mod state;
pub use state::AppState;
```

- [ ] **Step 4: Commit**

```bash
git add crates/spyder-gui
git commit -m "feat(gui): DTOs and AppState"
```

---

### Task 3: Venue TOML parse/emit helpers

**Files:**
- Create: `crates/spyder-gui/src/toml_venue.rs`
- Test: inline in `toml_venue.rs`

- [ ] **Step 1: Failing test — round-trip anchors**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Vec3;

    #[test]
    fn emit_and_parse_four_anchors() {
        let anchors = vec![
            Vec3::new(5.0, 3.0, 8.0),
            Vec3::new(-5.0, 3.0, 8.0),
            Vec3::new(-5.0, -3.0, 8.0),
            Vec3::new(5.0, -3.0, 8.0),
        ];
        let toml = emit_venue_toml(&anchors, &[], true, Vec3::new(0.0, 0.0, 1.5));
        let (robot, home) = parse_venue_toml(&toml).unwrap();
        assert_eq!(robot.anchors.len(), 4);
        assert!((home.z - 1.5).abs() < 1e-9);
    }
}
```

- [ ] **Step 2: Implement `emit_venue_toml` / `parse_venue_toml`**

Reuse the same line-oriented approach as `spyder-cli` `robot_from_toml` (copy into this module for now; dedupe later). Prefer calling `spyder_runtime::venue_toml_from_anchors` for emit when attachments empty.

- [ ] **Step 3: `cargo test -p spyder-gui` passes**

- [ ] **Step 4: Commit**

```bash
git commit -am "feat(gui): venue TOML parse/emit"
```

---

### Task 4: Health + venue + IK routes

**Files:**
- Create: `crates/spyder-gui/src/api.rs`
- Modify: `crates/spyder-gui/src/lib.rs`
- Modify: `crates/spyder-gui/src/main.rs`

- [ ] **Step 1: Router with `/health` and `/venue/from_preset`**

```rust
// api.rs sketch
use axum::{routing::{get, post}, Json, Router};
use std::sync::Arc;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/venue/from_preset", post(venue_from_preset))
        .route("/venue/load", post(venue_load))
        .route("/venue/toml", get(venue_toml))
        .route("/ik", post(ik))
        .with_state(state)
}

async fn health() -> Json<crate::dto::HealthResponse> {
    Json(crate::dto::HealthResponse {
        ok: true,
        version: crate::version().into(),
    })
}
```

Implement `venue_from_preset` to rebuild `Robot` from JSON `{kind:"rect", width, depth, height, point_mass}` and store in state.

Implement `ik` using `robot.ik` / `ik_with_options` when `mg` provided.

- [ ] **Step 2: Integration test**

```rust
#[tokio::test]
async fn health_ok() {
    let app = crate::api::router(AppState::new_rect());
    let resp = app
        .oneshot(
            http::Request::builder()
                .uri("/health")
                .body(http_body_util::Empty::<bytes::Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
```

Add `bytes` / `http` deps as needed for tests.

- [ ] **Step 3: main binds `0.0.0.0:7700` with CORS**

```rust
use tower_http::cors::CorsLayer;
let app = spyder_gui::api::router(AppState::new_rect()).layer(CorsLayer::permissive());
let listener = tokio::net::TcpListener::bind("127.0.0.1:7700").await.unwrap();
axum::serve(listener, app).await.unwrap();
```

- [ ] **Step 4: Manual smoke**

Run: `cargo run -p spyder-gui`  
In another shell: `curl -s localhost:7700/health`  
Expected: `{"ok":true,"version":"0.1.0"}`

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(gui): health, venue, ik HTTP routes"
```

---

### Task 5: FK, feasible, jacobian, workspace, traj, scene

**Files:**
- Create: `crates/spyder-gui/src/sim_svc.rs`
- Modify: `crates/spyder-gui/src/api.rs`

- [ ] **Step 1: Service helpers calling `spyder_sim`**

```rust
pub fn workspace_report(
    robot: &spyder_core::Robot,
    min: spyder_core::Vec3,
    max: spyder_core::Vec3,
    nx: usize, ny: usize, nz: usize,
    mg: f64, f_min: f64, f_max: f64,
) -> spyder_sim::WorkspaceReport {
    let box_ = spyder_sim::SampleBox { min, max, nx, ny, nz };
    let w = nalgebra::DVector::from_vec(vec![0.0, 0.0, -mg]);
    spyder_sim::sample_wrench_feasible(robot, &box_, w, f_min, f_max)
}
```

Wire routes from spec §6.

- [ ] **Step 2: Test workspace fraction > 0 for default rect**

```rust
#[tokio::test]
async fn workspace_has_feasible_points() { /* POST /workspace … assert fraction > 0 */ }
```

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(gui): sim routes workspace traj scene fk"
```

---

### Task 6: GUI-1 gate

- [ ] **Step 1: `cargo test -p spyder-gui` all green**
- [ ] **Step 2: Document curl examples in `docs/hardware.md` or README under “GUI API (dev)”**
- [ ] **Step 3: Commit + push branch `cursor/spyder-gui-55c0`**

**Stop here for review unless continuing to GUI-2.**

---

## Milestone GUI-2 — Design + Simulate UI

### Task 7: Vite React scaffold in `web/`

**Files:**
- Create: `web/package.json`, `web/vite.config.ts`, `web/index.html`, `web/src/main.tsx`, `web/src/App.tsx`, `web/src/styles.css`

- [ ] **Step 1: Scaffold**

```bash
cd web
npm create vite@latest . -- --template react-ts
npm install three @react-three/fiber @react-three/drei
npm install -D @types/three
```

Proxy `/api` → `http://127.0.0.1:7700` **or** call `http://127.0.0.1:7700` directly with CORS (already permissive).

- [ ] **Step 2: Brand chrome**

`App.tsx`: large **Spyder** wordmark, tabs Design | Simulate | Run, dark atmospheric CSS variables (not purple-default).

- [ ] **Step 3: Commit**

```bash
git add web
git commit -m "feat(gui): Vite React app scaffold"
```

---

### Task 8: API client + RobotScene

**Files:**
- Create: `web/src/api/client.ts`
- Create: `web/src/scene/RobotScene.tsx`

- [ ] **Step 1: `client.ts`**

```ts
const BASE = "http://127.0.0.1:7700";

export async function health() {
  const r = await fetch(`${BASE}/health`);
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function fromPreset(body: object) {
  const r = await fetch(`${BASE}/venue/from_preset`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export async function ik(xyz: [number, number, number], mg?: number) {
  const r = await fetch(`${BASE}/ik`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ xyz, mg }),
  });
  if (!r.ok) throw new Error(await r.text());
  return r.json() as Promise<{ lengths: number[]; tensions?: number[] }>;
}

// similarly: trajLine, workspace, sceneSnapshot, setAnchors, getToml
```

- [ ] **Step 2: `RobotScene.tsx`**

R3F canvas: anchors as spheres (draggable via `@react-three/drei` `PivotControls` or pointer events), dolly diamond, cable `Line` segments from anchor→dolly, Z-up (`camera` up `[0,0,1]` — set `THREE.Object3D.DEFAULT_UP`).

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(gui): API client and R3F robot scene"
```

---

### Task 9: Design page

**Files:**
- Create: `web/src/pages/DesignPage.tsx`

- [ ] **Step 1: On mount `fromPreset({kind:"rect", width:10, depth:6, height:8, point_mass:true})`**
- [ ] **Step 2: Inspector inputs bound to anchors; on blur POST `/venue/set_anchors`**
- [ ] **Step 3: Save button downloads TOML from `GET /venue/toml`**
- [ ] **Step 4: Load button reads file text → `POST /venue/load`**
- [ ] **Step 5: Commit**

```bash
git commit -am "feat(gui): Design tab venue edit load/save"
```

---

### Task 10: Simulate page

**Files:**
- Create: `web/src/pages/SimulatePage.tsx`

- [ ] **Step 1: Start/end XYZ fields + segments; button calls `/traj/line`**
- [ ] **Step 2: `requestAnimationFrame` or interval advances frame index; set dolly from waypoint; update cables via lengths or snapshot**
- [ ] **Step 3: Workspace button → `/workspace` → render feasible points as `Points`**
- [ ] **Step 4: Side readout of lengths/tensions from `/ik` with mg=9.81**
- [ ] **Step 5: Commit**

```bash
git commit -am "feat(gui): Simulate tab traj play and workspace"
```

---

## Milestone GUI-3 — Run

### Task 11: Run session on server

**Files:**
- Create: `crates/spyder-gui/src/run_svc.rs`
- Modify: `crates/spyder-gui/src/state.rs`, `api.rs`

- [ ] **Step 1: Enum `RunBackend` holding `Player<'static, …>` is awkward with lifetimes — instead keep `MockBackend` session struct:**

```rust
pub struct RunSession {
    pub backend_name: String,
    pub axes: Vec<spyder_runtime::Axis>,
    pub mock: Option<spyder_runtime::MockBackend>,
    pub player_home: spyder_core::Vec3,
    pub estopped: bool,
    // For MVP: only mock in-process; stepper later via MultiBoardBackend
}
```

Or store `Player` owned by reconstructing each play from robot+backend state (simpler): on `play_line`, build `Player::new(...).with_safety(...).move_line(...)`.

- [ ] **Step 2: Routes connect/disconnect/home/play_line/estop/status**
- [ ] **Step 3: Test mock play returns nonzero steps**
- [ ] **Step 4: Commit**

```bash
git commit -am "feat(gui): run session mock play and estop"
```

---

### Task 12: Run page UI

**Files:**
- Create: `web/src/pages/RunPage.tsx`

- [ ] **Step 1: Backend select default mock; Connect button**
- [ ] **Step 2: Large E-STOP control calling `/run/estop`**
- [ ] **Step 3: Play line uses same start/end as Simulate (lift shared store or React context)**
- [ ] **Step 4: Poll `/run/status` every 200ms while connected**
- [ ] **Step 5: Commit**

```bash
git commit -am "feat(gui): Run tab mock controls and estop"
```

---

### Task 13: Serve SPA from Axum + README + CI

**Files:**
- Modify: `crates/spyder-gui/src/main.rs`
- Modify: `README.md`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: `npm run build` in `web/`; serve `web/dist` with `tower_http::services::ServeDir` fallback to `index.html`**
- [ ] **Step 2: README**

```bash
# terminal A — API+UI (after web build) or API only
cargo run -p spyder-gui

# terminal B — Vite dev (optional)
cd web && npm run dev
```

Open `http://127.0.0.1:7700`

- [ ] **Step 3: CI job `gui` — `cargo test -p spyder-gui` + `cd web && npm ci && npm run build`**
- [ ] **Step 4: Final commit**

```bash
git commit -am "feat(gui): serve SPA, docs, CI"
```

---

## Self-review

| Spec requirement | Task |
|------------------|------|
| Design load/save/edit anchors | 3, 4, 9 |
| Simulate traj + workspace | 5, 10 |
| Run mock + estop | 11, 12 |
| Single process serve UI | 13 |
| No second IK | all call spyder-core |
| Acceptance curl/UI | 6, 13 |

Placeholder scan: none intentional. Types: `VenueDto`, `AppState`, `Vec3Dto` consistent across tasks.

---

## Execution handoff

**Plan complete and saved to:**
- `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`
- `docs/superpowers/plans/2026-07-10-spyder-gui.md`

**New chat — paste this:**

> Implement Spyder GUI from `docs/superpowers/specs/2026-07-10-spyder-gui-design.md` + `docs/superpowers/plans/2026-07-10-spyder-gui.md`. Branch `cursor/spyder-gui-55c0` from `main`. Use subagent-driven-development. Start Task 1. Do not reimplement IK. Land GUI-1 with tests before GUI-2.

**Two execution options in that chat:**

1. **Subagent-Driven (recommended)** — fresh subagent per task  
2. **Inline Execution** — executing-plans with checkpoints  
