# spyder

Parametric inverse kinematics for spider-cam / cable-driven camera robots.

Supports **N ≥ 3 motors**, rectangular and polygonal presets, irregular anchors, point-mass and rigid-platform modes, ideal / pulley / sag cable models, tension distribution, and winch→motor mapping.

## Quick start

```bash
cargo test
cargo run -p spyder-cli -- ik configs/rect_4.toml 0,0,2
```

Example FK round-trip:

```bash
cargo run -p spyder-cli -- ik configs/rect_4.toml 0.5,-0.2,2
# copy lengths, then:
cargo run -p spyder-cli -- fk configs/rect_4.toml 'L1,L2,L3,L4' 0,0,2
```

Python (PyO3) bindings are planned next on top of the same Rust crates.

## Workspace

| Crate | Role |
|-------|------|
| `spyder-core` | Pose, anchors, presets, IK/FK, `Robot` facade |
| `spyder-cables` | `Ideal`, `Pulley`, `Sag` (Irvine) models |
| `spyder-statics` | Structure matrix + closed-form tensions |
| `spyder-actuation` | Winch / motor step mapping |
| `spyder-cli` | `spyder ik` / `spyder fk` |

## Conventions

- World frame: right-handed, **Z-up**, meters
- Point-mass mode: cables meet at the dolly origin (classic Spidercam)
- Platform mode: per-cable body-frame attachment offsets + orientation

## Docs

- Design: `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`
- Plan: `docs/superpowers/plans/2026-07-10-spyder-phase1.md`

## License

MIT
