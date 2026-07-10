# Spyder GUI — Design / Simulate / Run

**Date:** 2026-07-10  
**Status:** Spec for implementation (handoff to new chat)  
**Depends on:** `main` @ `9a189e8` (core + sim + runtime shipped)  
**License:** MIT  

## 1. Intent

Ship an interactive **local GUI** that covers the product loop:

**Design venue → simulate motion/workspace → run (or dry-run) motors**

Today this loop is split across TOML editing, CLI, and static Plotly HTML. The GUI unifies it without reimplementing IK/FK/statics — it is a thin control surface over existing crates.

## 2. Goals & non-goals

### Goals (GUI MVP)

1. **Design**
   - Load / save venue TOML (presets + raw `[[anchors]]` / `[[attachments]]`)
   - Edit N≥3 anchors in a 3D viewport (drag) and numeric inspector
   - Toggle point-mass vs platform; set cable model (ideal / pulley / sag)
   - Set home pose; run field-cal-style “capture anchors → save venue”
2. **Simulate**
   - Live 3D scene: anchors, cables, dolly
   - Scrub / play Cartesian trajectories (line + waypoint list)
   - Overlay wrench-feasible workspace samples
   - Show live IK lengths, tensions, classification (IRPM/CRPM/RRPM)
3. **Run**
   - Backend picker: mock / stepper / ODrive / multiboard
   - Device + axis-map config
   - Play trajectory with safety limits visible; e-stop button
   - Closed-loop + realtime toggles; show feedback pose / steps
4. **Single local process** — one command starts API + UI; works offline after first asset load (vendor Three.js in `web/`)

### Non-goals (MVP)

- Cloud multi-user / accounts
- Full dynamics / vibration / vision servoing
- ROS2 bridge UI
- Mobile-first layout (desktop browser / desktop window is enough)
- Replacing CLI or Python bindings

## 3. Users & jobs

| User | Job |
|------|-----|
| Maker | Lay out 4 winches in a garage, dry-run path, then talk to Arduino sim / serial |
| Researcher | Compare ideal vs pulley, inspect tensions, export workspace |
| Operator | Load known venue, home, run a rehearsed line with e-stop ready |

## 4. Architecture decision

### Selected: **Local Axum API + Vite/React/Three.js SPA**

```
┌─────────────────────────────────────────────┐
│  Browser SPA (Vite + React + R3F/Three)     │
│  Design | Simulate | Run tabs               │
└──────────────────▲──────────────────────────┘
                   │ HTTP JSON (localhost)
┌──────────────────┴──────────────────────────┐
│  spyder-gui (Axum)                          │
│  wraps spyder-core / sim / runtime          │
│  optional: serve dist/ static files         │
└─────────────────────────────────────────────┘
```

**Why this over alternatives**

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| Axum + Vite SPA | Reuses Rust crates directly; easy CI; serial OK from server; matches current HTML viz habit | Two processes in dev | **MVP** |
| Tauri + React | Native window, easy packaging | Extra toolchain; slower first ship | Phase GUI-2 shell |
| egui only | Pure Rust | Weak 3D editing UX | Reject for primary UI |
| Python Gradio | Fast forms | Poor 3D design | Reject |

**Later:** wrap the same SPA in Tauri; call the same Axum port or replace HTTP with Tauri commands that call the same service layer.

### Service layer (shared)

Introduce `crates/spyder-gui` with pure service helpers used by Axum handlers — **no HTTP types in core math**.

```
crates/spyder-gui/          # Axum binary + service API
  src/
    main.rs                 # serve API + static
    api.rs                  # routes
    state.rs                # AppState (Robot, optional run session)
    dto.rs                  # request/response JSON
    design.rs               # venue mutate helpers
    sim_svc.rs              # scene / workspace / traj
    run_svc.rs              # backend connect / play / estop
web/                        # Vite React app
  src/
    App.tsx
    pages/{Design,Simulate,Run}.tsx
    scene/RobotScene.tsx    # R3F
    api/client.ts
```

## 5. Information architecture

### Top-level tabs (one composition per mode)

1. **Design** — venue geometry is the hero; inspector dock secondary  
2. **Simulate** — same 3D stage; timeline + readouts  
3. **Run** — same 3D stage; transport/safety controls dominant  

Shared: **inspector** (numbers), bottom **status bar** (backend, e-stop, residual).

### Design tab

- 3D: anchors as draggable handles; cables update live via IK at home or scrub pose  
- Inspector: preset buttons (rect / polygon / load file); per-anchor x,y,z; point_mass; model; home  
- Actions: Save TOML, Load TOML, Export calibration JSON  

### Simulate tab

- Pose scrubber / waypoint editor  
- Play/pause trajectory (precomputed `/traj` then client animate, or per-frame `/ik`)  
- Workspace sample button → point cloud overlay  
- Readouts: lengths[], tensions[], classify, feasible bool  

### Run tab

- Backend form: type, device, baud, axis-map JSON  
- Connect / Disconnect  
- Home, Play line, E-Stop (always visible when connected)  
- Toggles: closed-loop, realtime  
- Live: steps[], feedback pose, safety trip reason  

## 6. API surface (MVP)

Base: `http://127.0.0.1:7700`

| Method | Path | Body | Response |
|--------|------|------|----------|
| GET | `/health` | — | `{ok:true, version}` |
| POST | `/venue/load` | `{toml: string}` | `{venue, classify}` |
| POST | `/venue/from_preset` | `{kind, width?, depth?, height?, n?, radius?, point_mass}` | `{venue, classify}` |
| POST | `/venue/set_anchors` | `{anchors, attachments?, point_mass}` | `{venue, classify}` |
| GET | `/venue/toml` | — | `{toml}` |
| POST | `/ik` | `{xyz, model?, mg?}` | `{lengths, tensions?}` |
| POST | `/fk` | `{lengths, seed}` | `{xyz, orientation_rv, method, residual}` |
| POST | `/jacobian` | `{xyz}` | `{rows}` |
| POST | `/feasible` | `{xyz, mg?, f_min?, f_max?}` | `{ok}` |
| POST | `/workspace` | `{min, max, nx, ny, nz, mg, f_min, f_max}` | `{fraction, samples}` |
| POST | `/traj/line` | `{start, end, segments}` | `{waypoints, lengths}` |
| POST | `/scene/snapshot` | `{xyz}` | `{anchors, dolly, attachments, lengths}` |
| POST | `/run/connect` | `{backend, device?, baud?, axis_map?}` | `{ok, axes}` |
| POST | `/run/disconnect` | — | `{ok}` |
| POST | `/run/home` | — | `{ok}` |
| POST | `/run/play_line` | `{start, end, segments, closed_loop, realtime}` | `{final_steps, feedback_pose?}` |
| POST | `/run/estop` | — | `{ok}` |
| POST | `/run/clear_estop` | — | `{ok}` |
| GET | `/run/status` | — | `{connected, backend, estopped, steps?, pose?}` |

**DTO conventions:** meters, Z-up; JSON arrays for vectors; errors `{error: string}` with HTTP 4xx.

**State:** server holds current `Robot`, optional connected run session behind `tokio::sync::Mutex`. Single-operator local — no multi-session.

## 7. Frontend design rules

- First viewport of each tab = **one composition** (3D stage), not a widget dashboard  
- Brand **Spyder** is a hero-level mark in the chrome  
- Expressive fonts (not Inter/Roboto/Arial/system) — display + mono for numbers  
- Atmospheric background (subtle gradient / dark venue), not flat gray  
- No cards in the 3D hero; inspector panels only where interaction needs grouping  
- Motion: camera ease on tab enter; cable update lerp; e-stop attention pulse — ≥2 intentional motions  
- Avoid purple-glow / cream-serif / broadsheet clichés  

## 8. Safety & hardware UX

- Run tab defaults to **mock**  
- Connecting stepper/ODrive requires explicit Connect; Play disabled until connected  
- E-stop always visible when connected; calls server `Player::estop`  
- Soft limits shown numerically  
- Never auto-connect on load  

## 9. Testing strategy

- **Rust:** Axum route tests — load preset, ik, workspace fraction > 0  
- **Run mock:** connect → play_line → steps nonzero → estop latches  
- **Frontend:** Vitest for client parsers; Playwright: open Design, apply rect preset, Simulate play  
- **CI:** `cargo test -p spyder-gui`; `npm ci && npm run build` in `web/`  

## 10. Acceptance criteria (MVP done when)

1. `cargo run -p spyder-gui` serves UI at `http://127.0.0.1:7700`  
2. Create rect-4 venue, drag an anchor, save TOML, reload  
3. Simulate tab plays a line with moving dolly + cables; workspace overlay works  
4. Run tab mock-plays the same line and shows step counts  
5. E-stop on mock prevents further play until cleared  
6. No second IK implementation — all math via existing crates  
7. README documents one-command GUI start  

## 11. Phased delivery

| Phase | Deliverable |
|-------|-------------|
| **GUI-0** | Spec + plan (these docs) |
| **GUI-1** | Axum service + venue/ik/fk/workspace/traj routes + tests |
| **GUI-2** | React/R3F Design + Simulate tabs |
| **GUI-3** | Run tab + mock backend (stepper optional) |
| **GUI-4** | Playwright, polish, optional Tauri shell |

## 12. New-chat handoff prompt

Paste into a new agent chat:

> Implement the Spyder GUI per `docs/superpowers/specs/2026-07-10-spyder-gui-design.md` and `docs/superpowers/plans/2026-07-10-spyder-gui.md`. Start at Task 1 on a new branch `cursor/spyder-gui-55c0` from `main`. Do not reimplement IK — call existing crates. Ship GUI-1 (API green + tests) first, commit/push, then continue GUI-2 unless told to stop.

## 13. Decisions log

| Topic | Decision |
|-------|----------|
| UI host | Local Axum + Vite SPA |
| 3D | Three.js via React Three Fiber |
| Math | Existing spyder-* crates only |
| Hardware | Server-side runtime backends |
| Desktop shell | Optional Tauri later |
| Port | 7700 default |
