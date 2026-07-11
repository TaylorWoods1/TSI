# spyder

Parametric inverse kinematics for spider-cam / cable-driven camera robots.

Supports **N ≥ 3 motors**, rectangular and polygonal presets, irregular anchors, point-mass and rigid-platform modes, ideal / pulley / sag cable models, tension distribution, winch→motor mapping, workspace sampling + 3D HTML viz, trajectory playback, a local **Design / Simulate / Run GUI**, and Python bindings.

## Quick start (Rust)

```bash
cargo test --workspace
mkdir -p artifacts

cargo run -p spyder-gui   # API + UI at http://127.0.0.1:7700 (build web/ first)
cargo run -p spyder-cli -- ik configs/rect_4.toml 0,0,2
cargo run -p spyder-cli -- workspace configs/rect_4.toml artifacts/workspace_rect4
# open artifacts/workspace_rect4.html
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene.html
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene_anim.html \
  --to 0.5,0,1.5 --segments 12 --workspace
cargo run -p spyder-cli -- calibrate configs/rect_4.toml 0,0,1.5 artifacts/cal.json
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend mock --closed-loop --cal artifacts/cal.json

# Generate axis-map template before use:
cargo run -p spyder-cli -- axis-map-example configs/axis_map_dual_odrive.json

cargo run -p spyder-cli -- field-cal \
  "5,3,8;-5,3,8;-5,-3,8;5,-3,8" 0,0,1.5 artifacts/venue.toml
cargo run -p spyder-cli -- venue-from-cal artifacts/cal.json artifacts/venue_from_cal.toml
```

## Python

See [python/README.md](python/README.md).

```bash
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin==1.4.0 pytest
maturin develop --release
pytest tests/ -q
jupyter notebook ../notebooks/01_ik_workspace.ipynb
```

## Hardware

See [docs/hardware.md](docs/hardware.md) and [docs/cli.md](docs/cli.md#playback).

```bash
# Dry-run
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 --backend mock

# TCP firmware simulator
cargo run -p spyder-stepper-sim -- 9002
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend stepper --device 127.0.0.1:9002
```

Firmware: `firmware/spyder_stepper/spyder_stepper.ino`

## GUI

See [docs/gui.md](docs/gui.md) and [web/README.md](web/README.md).

```bash
cd web && npm ci && npm run build
cargo run -p spyder-gui
# open http://127.0.0.1:7700

# Dev: API + Vite hot reload
cargo run -p spyder-gui          # terminal 1
cd web && npm run dev            # terminal 2 → http://127.0.0.1:5173

# Desktop shell (Tauri)
cargo build -p spyder-gui && cd web && npm run build && cd ../apps/spyder-tauri
npm install && npm run tauri dev
```

See [docs/gui-configurator.md](docs/gui-configurator.md) and [docs/gui-tauri.md](docs/gui-tauri.md).

## Testing

```bash
cargo test --workspace     # Rust (107+ tests)
cargo test -p spyder-gui   # GUI API routes
cd web && npm test         # Vitest
cd python && pytest tests/ # Python bindings
```

## Crates

| Crate | Role |
|-------|------|
| `spyder-core` | Pose, anchors, presets, IK/FK, `Robot`, cable model kinds |
| `spyder-cables` | `Ideal`, `Pulley`, `Sag` (Irvine) |
| `spyder-statics` | Structure matrix + closed-form tensions |
| `spyder-actuation` | Winch / motor step mapping |
| `spyder-sim` | Workspace sampling, CSV/JSON/HTML, 3D scene, trajectories |
| `spyder-runtime` | Backends, `Player`, safety, calibration, axis map, feedback |
| `spyder-stepper-sim` | TCP firmware simulator |
| `spyder-cli` | `ik` / `fk` / `workspace` / `scene` / `calibrate` / `play` |
| `spyder-gui` | Local Design / Simulate / Run GUI (Axum + React) |
| `python/` | PyO3 bindings |

## Conventions

- World frame: right-handed, **Z-up**, meters
- Point-mass mode: cables meet at the dolly origin (classic Spidercam)
- Platform mode: per-cable body-frame attachment offsets + orientation
- Venue TOML schema: [docs/config-schema.md](docs/config-schema.md)

## Docs

| Document | Description |
|----------|-------------|
| [docs/README.md](docs/README.md) | Documentation index |
| [docs/architecture.md](docs/architecture.md) | Crate graph and data flow |
| [docs/gui.md](docs/gui.md) | GUI setup, API, testing |
| [docs/cli.md](docs/cli.md) | CLI command reference |
| [docs/hardware.md](docs/hardware.md) | Motors, protocol, calibration |
| [docs/config-schema.md](docs/config-schema.md) | Venue TOML fields |
| [web/README.md](web/README.md) | Frontend (Vite/React) |
| [python/README.md](python/README.md) | Python bindings |

Planning archive: `docs/superpowers/specs/` and `docs/superpowers/plans/`.

## License

MIT
