# spyder

Parametric inverse kinematics for spider-cam / cable-driven camera robots.

Supports **N ≥ 3 motors**, rectangular and polygonal presets, irregular anchors, point-mass and rigid-platform modes, ideal / pulley / sag cable models, tension distribution, winch→motor mapping, workspace sampling + 3D HTML viz, trajectory playback, and Python bindings.

## Quick start (Rust)

```bash
cargo test
cargo run -p spyder-cli -- ik configs/rect_4.toml 0,0,2
cargo run -p spyder-cli -- workspace configs/rect_4.toml artifacts/workspace_rect4
# open artifacts/workspace_rect4.html
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene.html
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene_anim.html \
  --to 0.5,0,1.5 --segments 12 --workspace
cargo run -p spyder-cli -- calibrate configs/rect_4.toml 0,0,1.5 artifacts/cal.json
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend mock --closed-loop --cal artifacts/cal.json
cargo run -p spyder-cli -- axis-map-example configs/axis_map_dual_odrive.json
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,1.5 0.2,0,1.5 8 \
  --backend mock --axis-map configs/axis_map_dual_odrive.json
```

## Python

```bash
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin==1.4.0
maturin develop --release
jupyter notebook ../notebooks/01_ik_workspace.ipynb
```

```python
from spyder import Robot
r = Robot.rect(10, 6, 8)
print(r.ik(0.5, -0.2, 2.0))
print(r.classify())                 # RRPM
print(r.is_feasible(0, 0, 2))
print(r.ik_tensions(0, 0, 2))
r.set_model("pulley", pulley_radius=0.05)
print(r.model(), r.ik(0, 0, 2)[0])
print(r.jacobian(0, 0, 2))
print(r.workspace_fraction(-2, 2, -2, 2, 0.5, 4, 6, 6, 5))
```

## Hardware

See [docs/hardware.md](docs/hardware.md).

```bash
# Dry-run
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 --backend mock

# TCP firmware simulator
cargo run -p spyder-stepper-sim -- 9002
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend stepper --device 127.0.0.1:9002

# Real Arduino steppers / ODrive
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend stepper --device /dev/ttyUSB0 --baud 115200
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 0.5,0,2 5 \
  --backend odrive --device /dev/ttyACM0
```

Firmware: `firmware/spyder_stepper/spyder_stepper.ino`

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
| `python/` | PyO3 bindings |

## Conventions

- World frame: right-handed, **Z-up**, meters
- Point-mass mode: cables meet at the dolly origin (classic Spidercam)
- Platform mode: per-cable body-frame attachment offsets + orientation

## Docs

- Design: `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`
- Plan: `docs/superpowers/plans/2026-07-10-spyder-phase1.md`

## License

MIT
