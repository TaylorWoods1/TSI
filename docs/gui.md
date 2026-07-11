# Spyder GUI

Local **Design → Simulate → Run** interface: Axum API on port **7700** + React/Three.js SPA.

**Full configurator guide:** [gui-configurator.md](gui-configurator.md)

## Quick start (bundled UI)

```bash
cd web && npm ci && npm run build
cargo run -p spyder-gui
# open http://127.0.0.1:7700
```

If `web/dist` is missing, the server still starts but only serves the API.

On Linux: `sudo apt install libudev-dev pkg-config` for serial backends.

## Development (hot reload)

Terminal 1 — API:

```bash
cargo run -p spyder-gui
```

Terminal 2 — Vite dev server (proxies API to `:7700`):

```bash
cd web && npm run dev
# open http://127.0.0.1:5173
```

## Tabs

| Tab | What it does |
|-----|----------------|
| **Design** | Rect/polygon presets, platform toggle, attachments, cable model, per-anchor pulley, home pose, motor mapping, field-cal, TOML |
| **Simulate** | Pose scrubber, waypoint editor, IK/FK/Jacobian/feasible panels (incl. motor steps), workspace overlay, Plotly export |
| **Run** | Mock / stepper / odrive / multiboard connect, play line & waypoints, E-stop, live 3D feedback |

## HTTP API

Base URL: `http://127.0.0.1:7700`

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness + version |
| GET | `/venue` | Current venue + classify |
| POST | `/venue/load` | Parse TOML into server state |
| POST | `/venue/from_preset` | Rect or polygon preset |
| POST | `/venue/set_anchors` | Replace anchors (rich `AnchorDto`) |
| POST | `/venue/home` | Set home pose |
| POST | `/venue/set_model` | Cable model + params |
| GET | `/venue/toml` | Export current venue |
| GET/POST | `/venue/motors` | Per-cable drum radius & steps/rev |
| POST | `/ik`, `/fk`, `/jacobian`, `/feasible` | Kinematics / analysis |
| POST | `/workspace` | Wrench-feasible samples |
| POST | `/traj/line`, `/traj/waypoints` | Cartesian trajectories |
| POST | `/scene/snapshot`, `/scene/export` | 3D scene JSON / Plotly HTML |
| GET/POST | `/calibration/*` | Field calibration (`GET /calibration/venue_toml` = export venue from cal) |
| POST | `/run/connect` | Connect motor backend (serial/TCP/odrive/multiboard) |
| POST | `/run/play_line`, `/run/play_waypoints` | Play trajectory |
| POST | `/run/estop` | Latch e-stop |

### Smoke tests

```bash
curl -s localhost:7700/health
curl -s localhost:7700/venue
curl -s -X POST localhost:7700/venue/from_preset \
  -H 'Content-Type: application/json' \
  -d '{"kind":"rect","width":10,"depth":6,"height":8,"point_mass":true}'
```

## Testing

```bash
cargo test -p spyder-gui
cd web && npm test && npm run build && npm run e2e
```

Playwright E2E covers rect preset, pulley model, simulate play, motor mapping, multiboard mock, and mock e-stop.

## See also

- [gui-configurator.md](gui-configurator.md) — full feature guide
- [gui-tauri.md](gui-tauri.md) — desktop shell (`apps/spyder-tauri`)

## Code map

| Path | Role |
|------|------|
| `crates/spyder-gui/` | Axum server, DTOs, calibration + run services |
| `web/src/hooks/useSceneSnapshot.ts` | Debounced shared scene snapshot |
| `web/src/pages/` | Design, Simulate, Run tabs |
| `web/src/scene/RobotScene.tsx` | R3F viewport (Z-up, model-aware cables) |
| `web/src/api/client.ts` | Fetch wrappers |

See [web/README.md](../web/README.md) for frontend details.
