# Spyder — What's Next (post Phase 1–3 bootstrap)

**Date:** 2026-07-10  
**Status on `main`:** Phase 1 core + early Phase 2 sim + early Phase 3 runtime are **merged**.  
**Spec:** `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`

## Done (shipped on main)

| Area | Status |
|------|--------|
| Phase 1 math core (IK/FK, Ideal/Pulley/Sag, statics closed-form, actuation) | Done |
| Python PyO3 + configs + notebook | Done (thin API) |
| Workspace sampling + Plotly HTML | Done |
| 3D scene (anchors/cables/dolly) | Done |
| Runtime Player + mock/stepper/ODrive | Done |
| Calibration, safety limits, e-stop, closed-loop FK | Done |
| Axis-map JSON + firmware/TCP sim | Done (map not yet driving multi-transport) |

## Gaps vs original spec

Still thin or missing relative to §6–14:

1. **Jacobian module** — length Jacobian / twist mapping not exposed as its own API  
2. **Restraint classification** — IRPM / CRPM / RRPM helper  
3. **QP/LP tension fallback** — only Pott closed-form today  
4. **Platform 6DOF FK** — translation-focused; full orientation FK incomplete  
5. **Python depth** — no pulley/sag/wrench/feasibility/tensions in bindings  
6. **CI** — no GitHub Actions yet  
7. **Multi-transport Player** — axis map prints devices; one transport per play session  
8. **Animated trajectory scene** — static pose HTML only  
9. **Repo rename** — GitHub still `TSI` (needs admin rename in UI)

## Recommended next phase: **Phase 4 — Harden & close the loop**

Priority order (highest leverage first):

### P0 — Make it trustworthy

1. **CI** — `cargo test` + Python `maturin develop` smoke on push  
2. **Property / golden tests** — transform invariance, N-gon IK, sag/tension regression fixtures  
3. **Mark Phase 1 plan checkboxes** / acceptance audit against spec §13  

### P1 — Finish research API surface

4. **Python parity** — expose models, wrench, tensions, `is_wrench_feasible`, platform mode  
5. **Jacobian + classification** — `J`, `Aᵀ`, IRPM/CRPM/RRPM  
6. **QP tension fallback** when closed-form fails bounds  

### P2 — Sim polish

7. **Animated scene** — scrub/play waypoints in Plotly (or simple Three.js)  
8. **Workspace + scene in one report** — feasible volume overlay with cables  

### P3 — Runtime for real hardware

9. **Multi-board Player** — open one transport per axis-map device; fan-out steps  
10. **Realtime loop** — fixed-rate tick, lookahead, soft-stop on safety trip  
11. **Field calibration UX** — measure anchors interactively; persist venue TOML  

### Later (explicit non-goals for now)

- Dynamics / vibration control  
- Vision-based feedback  
- ROS2 / C ABI  
- Full desktop GUI  

## Suggested first implementation slice

**“Harden + Python parity”** — CI + expand PyO3 to match Rust `Robot` (models, wrench, feasibility) + one Jacobian/classification module. Unblocks notebooks and research use without needing more hardware.

## Manual ops

- Rename GitHub repo `TSI` → `spyder` in Settings (token lacks admin)  
- Then: `git remote set-url origin https://github.com/TaylorWoods1/spyder.git`
