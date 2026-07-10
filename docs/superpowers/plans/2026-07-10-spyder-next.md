# Spyder — What's Next (post Phase 1–3 bootstrap)

**Date:** 2026-07-10  
**Status on `main`:** Phase 1 core + early Phase 2 sim + early Phase 3 runtime are **merged**.  
**Spec:** `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`

## Done (shipped on main)

| Area | Status |
|------|--------|
| Phase 1 math core (IK/FK, Ideal/Pulley/Sag, statics closed-form, actuation) | Done |
| Python PyO3 + configs + notebook | Done |
| Workspace sampling + Plotly HTML | Done |
| 3D scene (anchors/cables/dolly) | Done |
| Runtime Player + mock/stepper/ODrive | Done |
| Calibration, safety limits, e-stop, closed-loop FK | Done |
| Axis-map JSON + firmware/TCP sim | Done (map not yet driving multi-transport) |

## Phase 4 slice (this branch)

| Item | Status |
|------|--------|
| GitHub Actions CI (`cargo test` + Python smoke) | Done |
| Length Jacobian (`Robot::length_jacobian`) | Done |
| IRPM / CRPM / RRPM classification | Done |
| QP/projected-gradient tension fallback | Done |
| Python parity (model, tensions, feasible, classify, jacobian) | Done |
| Property tests (rigid translation, N-gon round-trip) | Done |

## Remaining gaps

1. **Platform 6DOF FK** — translation-focused; full orientation FK incomplete  
2. **Multi-transport Player** — axis map prints devices; one transport per play session  
3. **Animated trajectory scene** — static pose HTML only  
4. **Repo rename** — GitHub still `TSI` (needs admin rename in UI)

## Recommended next after this PR

### P2 — Sim polish
- Animated scene (scrub/play waypoints)
- Workspace + scene overlay

### P3 — Runtime for real hardware
- Multi-board Player from axis map
- Realtime fixed-rate loop + soft-stop
- Field calibration UX → venue TOML

### Later
- Dynamics / vibration / vision / ROS2 / C ABI

## Manual ops

- Rename GitHub repo `TSI` → `spyder` in Settings  
- Then: `git remote set-url origin https://github.com/TaylorWoods1/spyder.git`
