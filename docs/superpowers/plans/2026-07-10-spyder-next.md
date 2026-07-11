# Spyder — What's Next

**Date:** 2026-07-11  
**Spec:** `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`

## Shipped (on `main`)

| Area | Status |
|------|--------|
| Core / sim / runtime / CLI | Done |
| Model-aware IK/FK (ideal / pulley / sag) | Done |
| Platform 6-DOF FK + field calibration | Done |
| **Full GUI configurator** (Design / Simulate / Run) | Done — [docs/gui-configurator.md](../../gui-configurator.md) |
| Hardware in GUI (stepper serial+TCP, ODrive, multiboard) | Done |
| Motor mapping, calibration export, play waypoints | Done |
| Playwright E2E + CI (Rust, web, Python) | Done |
| **Tauri desktop shell** (scaffold) | Done — [docs/gui-tauri.md](../../gui-tauri.md) |

## Next (optional polish)

| Priority | Item |
|----------|------|
| Desktop | Bundle `spyder-gui` binary in Tauri release; tray icon; single-instance lock |
| UX | TransformControls rotation gizmo; waypoint drag-reorder |
| Field | E2E against live `spyder-stepper-sim` TCP in CI (optional) |
| Ops | GitHub rename `TSI` → `spyder` (admin UI), if desired |

Specs: `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`  
Archive: `docs/superpowers/plans/2026-07-10-spyder-full-configurator.md` (completed)
