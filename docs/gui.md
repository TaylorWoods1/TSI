# Spyder GUI

Local **Design → Simulate → Run** interface: Axum API on port **7700** + React/Three.js SPA.

## Quick start (bundled UI)

```bash
cd web && npm ci && npm run build
cargo run -p spyder-gui
# open http://127.0.0.1:7700
```

If `web/dist` is missing, the server still starts but only serves the API (see curl examples below).

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
| **Design** | Rect-4 preset, drag/edit anchors, save/load venue TOML |
| **Simulate** | Line trajectory play, workspace overlay, IK/tension readout |
| **Run** | Mock backend connect, play line, E-stop (stepper/ODrive planned) |

### MVP limitations

| Feature | Status |
|---------|--------|
| Mock motor playback | Shipped |
| Stepper / ODrive / multiboard in GUI | CLI only; GUI rejects non-`mock` |
| Platform mode toggle in Design | CLI/TOML; UI uses point-mass preset |
| Cable model picker (pulley/sag) | API supports; UI not wired |
| Field-cal / calibration export | CLI only |

## HTTP API

Base URL: `http://127.0.0.1:7700`

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness + version |
| POST | `/venue/load` | Parse TOML into server state |
| POST | `/venue/from_preset` | Rect or polygon preset |
| POST | `/venue/set_anchors` | Replace anchor list |
| GET | `/venue/toml` | Export current venue |
| POST | `/ik`, `/fk`, `/jacobian`, `/feasible` | Kinematics |
| POST | `/workspace` | Wrench-feasible samples |
| POST | `/traj/line` | Cartesian line + IK lengths |
| POST | `/scene/snapshot` | 3D scene JSON |
| POST | `/run/connect` | Connect mock backend |
| POST | `/run/play_line` | Play trajectory |
| POST | `/run/estop` | Latch e-stop |

Full route table: `docs/superpowers/specs/2026-07-10-spyder-gui-design.md` §6.

### Smoke tests

```bash
curl -s localhost:7700/health
curl -s -X POST localhost:7700/venue/from_preset \
  -H 'Content-Type: application/json' \
  -d '{"kind":"rect","width":10,"depth":6,"height":8,"point_mass":true}'
curl -s -X POST localhost:7700/ik \
  -H 'Content-Type: application/json' \
  -d '{"xyz":[0,0,2]}'
```

## Testing

```bash
# Rust API integration tests
cargo test -p spyder-gui

# Frontend unit tests (API client)
cd web && npm test
```

Playwright E2E is planned but not yet in CI.

## Code map

| Path | Role |
|------|------|
| `crates/spyder-gui/` | Axum server, DTOs, service layer |
| `web/src/pages/` | Design, Simulate, Run tabs |
| `web/src/scene/RobotScene.tsx` | R3F viewport (Z-up) |
| `web/src/api/client.ts` | Fetch wrappers |

See [web/README.md](../web/README.md) for frontend details.
