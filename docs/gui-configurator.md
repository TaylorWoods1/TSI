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
| **Field calibration** | Capture at home, per-anchor measure, export JSON + **venue TOML**, load JSON, apply to venue |
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
| `stepper` | Serial path (`/dev/ttyUSB0`) or TCP `host:port` |
| `odrive` | Serial or TCP; enters closed-loop on connect |
| `multiboard` | `axis_map` JSON (`{ "cables": [{ device, baud, axis, steps_per_rev }] }`); optional **Mock hardware** dry-run |

Always **Connect** explicitly before Play. Calibration **home lengths** apply automatically on connect. **Play waypoints** replays the Simulate trajectory. **E-stop** latches until cleared. Status bar shows model, classify, backend, FK residual.

### Stepper TCP example

```bash
# terminal A — stepper sim
cargo run -p spyder-stepper-sim

# terminal B — GUI
cargo run -p spyder-gui
# Run tab → backend stepper → device 127.0.0.1:5555 → Connect
```

### Multiboard axis map example

```json
{
  "cables": [
    { "device": "/dev/ttyACM0", "baud": 115200, "axis": 0, "steps_per_rev": 200 },
    { "device": "/dev/ttyACM0", "baud": 115200, "axis": 1, "steps_per_rev": 200 },
    { "device": "/dev/ttyACM1", "baud": 115200, "axis": 0, "steps_per_rev": 200 },
    { "device": "/dev/ttyACM1", "baud": 115200, "axis": 1, "steps_per_rev": 200 }
  ]
}
```

Enable **Mock hardware** to dry-run without opening serial ports.

## Motor mapping (Design tab)

Per-cable **drum radius** and **steps/rev** feed IK motor-command readouts (Simulate) and Run playback axes.

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/venue/motors` | Current per-cable mapping |
| POST | `/venue/motors` | Replace mapping (`{ "axes": [{ drum_radius_m, steps_per_rev }] }`) |

## HTTP API (highlights)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/venue` | Current venue + classify |
| POST | `/venue/home` | Set home pose |
| GET/POST | `/venue/motors` | Per-cable drum/steps mapping |
| POST | `/traj/waypoints` | IK along waypoint list |
| POST | `/scene/export` | Plotly HTML |
| GET/POST | `/calibration/*` | Field-cal capture/apply; `GET /calibration/venue_toml` exports merged venue |
| POST | `/run/connect` | mock / stepper / odrive / multiboard |
| POST | `/run/play_waypoints` | Play waypoint list on connected backend |

Full table: `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`

## Desktop shell

A Tauri desktop wrapper ships as an installable `.deb` (Linux) with bundled `spyder-gui` + web assets — see [gui-tauri.md](gui-tauri.md) for dev and release builds.

## Testing

```bash
cargo test -p spyder-core -p spyder-cables -p spyder-sim -p spyder-gui
cd web && npm ci && npm test && npm run build && npm run e2e
```

## See also

- [docs/gui.md](gui.md) — quick reference
- [docs/superpowers/plans/2026-07-10-spyder-full-configurator.md](superpowers/plans/2026-07-10-spyder-full-configurator.md) — implementation plan
