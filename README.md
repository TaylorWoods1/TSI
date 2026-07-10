# spyder

Parametric inverse kinematics for spider-cam / cable-driven camera robots.

Supports **N ≥ 3 motors**, rectangular and polygonal presets, irregular anchors, point-mass and rigid-platform modes, ideal / pulley / sag cable models, tension distribution, winch→motor mapping, workspace sampling + 3D HTML viz, trajectory playback, and Python bindings.

## Quick start (Rust)

```bash
cargo test
cargo run -p spyder-cli -- ik configs/rect_4.toml 0,0,2
cargo run -p spyder-cli -- workspace configs/rect_4.toml artifacts/workspace_rect4
# open artifacts/workspace_rect4.html
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8
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
print(r.workspace_fraction(-2, 2, -2, 2, 0.5, 4, 6, 6, 5))
```

## Crates

| Crate | Role |
|-------|------|
| `spyder-core` | Pose, anchors, presets, IK/FK, `Robot`, cable model kinds |
| `spyder-cables` | `Ideal`, `Pulley`, `Sag` (Irvine) |
| `spyder-statics` | Structure matrix + closed-form tensions |
| `spyder-actuation` | Winch / motor step mapping |
| `spyder-sim` | Workspace sampling, CSV/JSON/HTML export, trajectories |
| `spyder-runtime` | `MotorBackend`, `MockBackend`, `StepperStub`, `Player` |
| `spyder-cli` | `ik` / `fk` / `workspace` / `play` |
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
