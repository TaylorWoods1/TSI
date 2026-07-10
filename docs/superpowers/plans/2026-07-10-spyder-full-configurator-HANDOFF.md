# HANDOFF — Spyder Full Configurator & Visualiser

**Date:** 2026-07-10  
**Plan:** `docs/superpowers/plans/2026-07-10-spyder-full-configurator.md`  
**Spec:** `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`

---

## Prerequisites on `main`

1. Math audit merged (`872774a` — model-aware IK/FK/pulley/sag/statics)
2. **Merge first:** `cursor/gui-cable-model-viz-fee8` → `main`  
   (model-aware `cable_paths`, Design cable picker, scene snapshot, pull arrows)

---

## Paste this as the new chat’s first message

```
Implement the Spyder FULL configurator and visualiser — all phases (0–5).

Read and follow exactly:
- docs/superpowers/plans/2026-07-10-spyder-full-configurator.md  (task checklists)
- docs/superpowers/specs/2026-07-10-spyder-gui-design.md           (product intent)
- docs/gui.md                                                      (update when done)

Branch: create `cursor/full-configurator-fee8` from `main` (after merging `cursor/gui-cable-model-viz-fee8` if not already on main).

Skills: subagent-driven-development or executing-plans — work phase-by-phase.

RULES:
- Do NOT reimplement IK/FK/statics — call spyder-core, spyder-sim, spyder-runtime only.
- Commit + push after each phase; open a draft PR after Phase 1; update PR through Phase 5.
- Run verification before claiming done:
    cargo test -p spyder-core -p spyder-cables -p spyder-sim -p spyder-gui
    cd web && npm ci && npm test && npm run build && npm run e2e
  (spyder-gui needs libudev-dev on Linux for serial backends)
- Match existing UI conventions: Z-up R3F scene, inspector sidebar, dark venue aesthetic.

PHASES (in order):
0. Prep — merge viz branch, shared useSceneSnapshot hook in web context
1. Venue configurator — platform toggle, attachments editor, home pose, polygon preset,
   per-anchor pulley fields (axis/radius/winch/runout), full TOML round-trip, AnchorDto
2. Simulate depth — pose scrubber, waypoint editor (/traj/waypoints), IK/FK/Jacobian/
   feasible/classify panels, platform 6-DOF orientation, configurable workspace box
3. Run & hardware — enable stepper/odrive/multiboard in run_svc (remove mock-only guard),
   connection form, live status poll → 3D feedback pose, e-stop UX
4. Calibration & export — /calibration/* API wrapping spyder-runtime::Calibration,
   field-cal UI in Design, motor mapping advanced panel, POST /scene/export Plotly HTML
5. Polish — status bar, GET /venue, Playwright E2E, docs/gui-configurator.md, update docs/gui.md

Definition of done: full checklist at bottom of the plan file.

Start at Phase 0 Task 0.1. Do not skip phases unless a phase is already complete on main.
When finished, mark PR ready for review with a summary of what shipped per phase.
```

---

## Context snapshot (for the agent)

| Item | Value |
|------|-------|
| Repo | `TaylorWoods1/TSI` (workspace: spyder CDPR) |
| GUI stack | Axum `spyder-gui` :7700 + Vite/React/R3F `web/` |
| Math crates | `spyder-core`, `spyder-cables`, `spyder-statics`, `spyder-sim` |
| Runtime | `spyder-runtime` — MockBackend, StepperBackend, ODriveBackend, MultiBoardBackend, Calibration |
| Viz branch | `cursor/gui-cable-model-viz-fee8` @ `79c2713` |
| Math on main | `872774a` |

### Already shipped (viz branch — do not redo)

- `spyder-core/src/cable_path.rs` — ideal/pulley/sag polylines
- `POST /venue/set_model`, extended `POST /scene/snapshot`
- Design: cable model picker, pulley radius, sag μ/EA
- RobotScene: polyline cables, pulley torus, pull arrows
- Simulate: model-aware IK readout, FK check, pull toggle

### Key gaps (this handoff)

- Platform mode + attachments UI
- Per-anchor pulley axis/winch/runout UI
- Home pose editor, polygon preset
- Jacobian/feasible/classify panels
- Waypoint editor, 6-DOF orientation scrub
- Real hardware Run (stepper/odrive/multiboard)
- Field calibration GUI
- Playwright E2E

### Key code entry points

```
crates/spyder-gui/src/api.rs          # routes
crates/spyder-gui/src/design.rs       # venue mutations
crates/spyder-gui/src/sim_svc.rs     # ik/fk/scene
crates/spyder-gui/src/run_svc.rs      # mock-only today — extend here
crates/spyder-runtime/src/calibration.rs
web/src/pages/{Design,Simulate,Run}Page.tsx
web/src/scene/RobotScene.tsx
web/src/context.tsx
```

---

## Success = plan “Definition of done” section

All checkboxes in `2026-07-10-spyder-full-configurator.md` satisfied + CI green.
