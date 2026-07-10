# Spyder GUI — Full Configurator Guide

Interactive **Design → Simulate → Run** workflow for cable-driven robots. All kinematics/statics/runtime logic lives in `spyder-*` crates; the GUI is a thin Axum + React control surface.

## Quick start

```bash
cd web && npm ci && npm run build
cargo run -p spyder-gui
# open http://127.0.0.1:7700
```

Development with hot reload:

```bash
# terminal A
cargo run -p spyder-gui

# terminal B
cd web && npm run dev
# open http://127.0.0.1:5173
```

On Linux, install `libudev-dev` for serial backends (`sudo apt install libudev-dev`).

## Design tab

Configure the venue without editing TOML by hand.

| Feature | Description |
|---------|-------------|
| **Rect / polygon presets** | Rect: width, depth, height. Polygon: n cables, circumradius, height |
| **Point mass vs platform** | Toggle rigid platform; attachments editor when platform mode |
| **Cable model** | Ideal, pulley (radius), sag (μ, EA) |
| **Per-anchor pulley** | Radius override, axis preset (Z/X/Y), winch exit, runout |
| **Home pose** | XYZ fields; “Set home from dolly” |
| **Field calibration** | Capture at home, per-anchor measure, export/load JSON, apply to venue |
| **TOML** | Save / load full venue round-trip |

### TOML fields

```toml
point_mass = true
cable_model = "pulley"    # ideal | pulley | sag
pulley_radius = 0.06
sag_mu = 1.0
sag_ea = 1000000.0

[[anchors]]
x = 5.0
y = 3.0
z = 8.0
pulley_radius = 0.08      # optional per-anchor

[[attachments]]
x = 0.1
y = 0.0
z = 0.0

[home]
x = 0.0
y = 0.0
z = 2.0
```

## Simulate tab

| Feature | Description |
|---------|-------------|
| **Pose scrubber** | XYZ sliders (+ orientation when platform mode) |
| **Waypoint editor** | Table add/remove; import/export JSON; play full path |
| **Analysis panels** | IK, FK, Jacobian, feasible, classify |
| **Workspace** | Configurable sample box; feasible point cloud overlay |
| **Export** | Download Plotly HTML of current pose (pulley polylines) |

## Run tab

| Backend | Connect params |
|---------|----------------|
| `mock` | No device |
| `stepper` | TCP `host:port` (e.g. `127.0.0.1:8080` for stepper sim) |
| `odrive` | Use CLI for serial; GUI shows guidance |
| `multiboard` | Axis-map JSON (dry-run mock fan-out) |

Always **Connect** explicitly before Play. **E-stop** latches until cleared. Status bar shows model, classify, backend, FK residual.

### Stepper TCP example

```bash
# terminal A — stepper sim
cargo run -p spyder-stepper-sim

# terminal B — GUI
cargo run -p spyder-gui
# Run tab → backend stepper → device 127.0.0.1:8080 → Connect
```

## HTTP API (highlights)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/venue` | Current venue + classify |
| POST | `/venue/home` | Set home pose |
| POST | `/traj/waypoints` | IK along waypoint list |
| POST | `/scene/export` | Plotly HTML |
| GET/POST | `/calibration/*` | Field-cal capture/apply |
| POST | `/run/connect` | mock / stepper / multiboard |

Full table: `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`

## Testing

```bash
cargo test -p spyder-core -p spyder-cables -p spyder-sim -p spyder-gui
cd web && npm ci && npm test && npm run build && npm run e2e
```

## See also

- [docs/gui.md](gui.md) — quick reference
- [docs/superpowers/plans/2026-07-10-spyder-full-configurator.md](superpowers/plans/2026-07-10-spyder-full-configurator.md) — implementation plan
