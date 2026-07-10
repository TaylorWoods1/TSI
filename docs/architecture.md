# Architecture

Spyder is a Rust workspace for **cable-driven parallel robots** (spider-cam / winch rigs) with optional Python bindings and a local web GUI.

## World conventions

- Right-handed frame, **Z-up**, distances in **meters**
- **Point-mass mode:** all cables attach at the dolly origin (classic spider-cam)
- **Platform mode:** per-cable body-frame attachment offsets + 6-DOF pose

## Crate dependency graph

```
                    ┌─────────────┐
                    │ spyder-cli  │  binary: ik, fk, play, scene, …
                    │ spyder-gui  │  binary: Axum API + static web/dist
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
  ┌─────────────┐   ┌─────────────┐   ┌──────────────┐
  │ spyder-sim  │   │spyder-runtime│   │ (web SPA)    │
  │ workspace,  │   │ Player,      │   │ Vite/React   │
  │ scene HTML  │   │ backends     │   │ @ :7700      │
  └──────┬──────┘   └──────┬───────┘   └──────────────┘
         │                 │
         └────────┬────────┘
                  ▼
           ┌─────────────┐
           │ spyder-core │  Robot, IK/FK, presets, Jacobian
           └──────┬──────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
spyder-cables  spyder-statics  spyder-actuation
 Ideal/Pulley/  tensions,       winch → motor steps
 Sag models     structure A
```

Supporting crates:

| Crate | Role |
|-------|------|
| `spyder-stepper-sim` | TCP simulator for Arduino stepper firmware protocol |
| `python/` (PyO3) | `Robot` class wrapping `spyder-core` + `spyder-sim` |

## Layer responsibilities

### `spyder-core`

Single source of truth for kinematics. `Robot` combines anchors, attachments, cable model, and calls into:

- `spyder-cables` for length models (ideal / pulley / sag)
- `spyder-statics` for wrench feasibility and tensions
- Internal IK/FK solvers (analytic where possible, numeric fallback)

**Rule:** GUI, CLI, and Python must call `spyder-core` — no duplicate IK implementations.

### `spyder-sim`

Offline analysis: workspace sampling, trajectory waypoints, Plotly/HTML scene export.

### `spyder-runtime`

Hardware-facing layer: `Player` trajectory playback, `SafetyLimits`, motor backends (mock, stepper, ODrive, multiboard), calibration JSON, axis maps.

### `spyder-gui`

Local HTTP service on `127.0.0.1:7700`:

- JSON API wrapping core/sim/runtime (`crates/spyder-gui/src/api.rs`)
- Serves built SPA from `web/dist` (fallback to `index.html`)

State: one `Robot` + optional mock run session (single-operator local).

### `web/`

React Three Fiber viewport + inspector panels. Talks to the API via `src/api/client.ts`. Vite dev server proxies API routes to `:7700`.

## Data flow (GUI loop)

```
Design tab  → POST /venue/*     → Robot in AppState
Simulate    → POST /ik, /workspace, /traj/line
Run tab     → POST /run/*       → Player + MockBackend (MVP)
```

## Python

Built with maturin (`cd python && maturin develop`). The extension module `spyder` exposes `Robot` with methods mirroring common CLI workflows (IK, FK, workspace fraction, line IK).

## Firmware

`firmware/spyder_stepper/spyder_stepper.ino` implements the line protocol consumed by `spyder-runtime::StepperBackend`. Use `spyder-stepper-sim` for local TCP testing without hardware.
