# Spyder — Parametric Cable-Camera Inverse Kinematics Suite

**Date:** 2026-07-10  
**Status:** Draft for review  
**Repo:** `spyder` (formerly TSI)  
**License:** MIT (default unless changed before first public release)

## 1. Intent

Build a parametric inverse-kinematics suite for spider-cam / cable-driven parallel camera robots that supports **3, 4, 5, or N motors** in **arbitrary angular layouts**. The long-term product is a full research → design → plan → simulate → run toolchain. Delivery is phased:

| Phase | Focus | Packages |
|-------|--------|----------|
| **1** | Core IK / FK / statics / actuation | `spyder-core`, `spyder-cables`, `spyder-statics`, `spyder-actuation`, `spyder-py` |
| **2** | Simulation & visualization | `spyder-sim` |
| **3** | Hardware runtime | `spyder-runtime` |

This spec defines **Phase 1** in full and only the **interfaces/hooks** needed so Phases 2–3 plug in without rewriting the math core.

## 2. Goals & non-goals

### Goals (Phase 1)

- Parametric robot definition: N ≥ 3 cables, presets + raw anchors
- Dual end-effector modes: point-mass (3 DOF) and rigid platform (6 DOF)
- Cable models: ideal straight, swivel-pulley, Irvine sag (static catenary)
- Inverse kinematics for all models; forward kinematics numerical + analytic fast paths
- Tension distribution and wrench-feasibility checks
- Winch and motor command mapping (lengths → angles → steps/encoder units)
- Rust math core with Python (PyO3) API, CLI, example configs, notebooks
- Test suite with geometric fixtures, round-trip properties, and golden layouts

### Non-goals (Phase 1)

- Real-time motor drivers / firmware (Phase 3)
- Full 3D GUI simulator (Phase 2)
- Dynamics (inertia, Coriolis), vibration control, vision-based feedback
- ROS2 nodes (may consume C ABI later; not required now)
- Dual maintenance of a second IK implementation in C++

## 3. Background (research synthesis)

### Domain

Commercial **Spidercam / Skycam** systems are cable-driven parallel robots (CDPRs): typically four winches at venue corners, cables through corner pulleys to a dolly. Translation is controlled by cable lengths; camera pan/tilt is a separate stabilized head. Twin-cable variants use eight cables.

Maker projects (Marginally Clever, MaltMover, SpiderRobot, CableCamera, etc.) almost universally use **ideal Euclidean IK**:

\[
L_i = \|P - A_i\|
\]

Academic CDPR literature (Pott, Merlet, Gouttefarde, Carricato, Verhoeven, CASPR) adds:

- **Classification** by cables \(m\) vs DOF \(n\): IRPM / CRPM / RRPM (Ming–Higuchi / Verhoeven)
- **Structure (wrench) matrix** \(A^T\) mapping tensions to platform wrench
- **Pulley kinematics** (tangent + wrap arc)
- **Sagging cables** via Irvine’s elastic catenary (IK becomes kineto-static; underconstrained for \(m > 6\))
- **Tension distribution**: closed-form (Pott) and QP/LP with \(f_{\min}, f_{\max}\)

Open research platforms: **CASPR** (MATLAB), WireX, various ROS CDPR stacks. Spyder targets the same scientific capability with a camera-robot-first API and a Rust/Python maker-friendly path.

### Design implication

Spyder must be **as simple as a garage Spidercam** for the default path and **as capable as a research CDPR stack** when advanced models are selected — without forking the API.

## 4. Architecture

### 4.1 Approach

**Layered Cargo workspace** (selected over monolithic crate or Python-first rewrite):

```
spyder/
  crates/
    spyder-core/        # frames, pose, anchors, presets, ideal IK/FK orchestration
    spyder-cables/      # CableModel: Ideal, Pulley, Sag(Irvine)
    spyder-statics/     # structure matrix, tension solve, feasibility
    spyder-actuation/   # winch + motor mapping
  python/
    spyder/             # PyO3 bindings, CLI, config loaders
  configs/              # example venue layouts
  notebooks/            # research demos
  docs/
```

Later phases add `spyder-sim` and `spyder-runtime` that depend on the same traits; they do not reimplement IK.

### 4.2 Native + Python strategy

- **Rust** owns all numeric kinematics/statics (single source of truth)
- **Python** owns research UX: configs, CLI, notebooks, future sim glue
- Bindings via **PyO3 / maturin**
- Optional **C ABI** later for ROS/C++ consumers — not a second math core

### 4.3 Coordinate conventions

- World frame: **right-handed, Z-up**, meters
- Orientation: unit **quaternion** internally; Euler/RPY accepted at API edges
- Base cable exit points \(a_i\) in world frame
- Platform attachment points \(b_i\) in body frame
- Point-mass mode: all \(b_i = 0\) (or `point_mass=true`); pose is translation-only for IK inputs (orientation ignored or identity)

## 5. Core data model

| Type | Responsibility |
|------|----------------|
| `Frame` | World/body frame metadata and transforms |
| `Pose` | \(p \in \mathbb{R}^3\), \(R \in SO(3)\) |
| `Anchor` | Exit point \(a_i\); optional pulley axis + radius |
| `PlatformAttachment` | Body point \(b_i\) |
| `CableSpec` | Pair `(Anchor, PlatformAttachment)` + model params |
| `LayoutPreset` | `rect`, `regular_polygon(n, radius, height)`, `triangle`, … → anchors |
| `Robot` | N cables, cable model choice, winches/motors, mode flags |
| `IkResult` | lengths, winch angles, motor commands, optional tensions, diagnostics |
| `FkResult` | pose, residual, iterations, method used (`analytic` / `numeric`) |
| `Wrench` | force + torque on platform (e.g. gravity) |
| `Winch` / `Motor` | drum geometry, steps/rev, gearing, limits, direction sign |

**Layout specification (both):**

1. **Presets** for common venues (rectangle, regular N-gon at height, etc.)
2. **Raw anchors** for irregular / angular configurations, with optional per-motor overrides after a preset

## 6. Kinematics

### 6.1 Inverse kinematics

For each cable \(i\):

1. World-side attachment: \(B_i = p + R b_i\) (or \(B_i = p\) in point-mass mode)
2. Length from active `CableModel`:
   - **Ideal:** \(L_i = \|B_i - a_i\|\)
   - **Pulley:** length = free-span tangent + pulley wrap arc (swivel plane from pulley axis and \(B_i\))
   - **Sag:** Irvine model yields unstrained length \(L_0\) given horizontal/vertical force components; may require coupling to tension solve (kineto-static). Result type distinguishes geometric length vs unstrained length.

IK for ideal and pulley is **closed-form** given pose. Sag may be iterative per cable or joint with statics.

### 6.2 Forward kinematics

- **Default:** nonlinear least squares \(\min \|IK(pose) - L_{\mathrm{meas}}\|^2\) (Levenberg–Marquardt / Gauss–Newton), with numeric or analytic Jacobian
- **Analytic / reduced fast paths** (Phase 1 must ship both):
  1. **3-cable point-mass:** solve for position from three sphere intersections (two intersections → pick by gravity / workspace heuristic documented in code)
  2. **4-cable rectangular point-mass** with coincident anchors: reduced closed-form or dimension-reduced numeric solver specialized to axis-aligned rectangle footprints
- Fallback: always numerical FK for arbitrary N, non-rectangular layouts, and platform mode
- Report method used and residual in `FkResult`
- Dispatch rule: use a fast path only when robot geometry matches that path’s preconditions; otherwise numerical

### 6.3 Jacobian & structure matrix

- Length Jacobian: \(\dot{L} = J\, t\) (platform twist \(t\))
- Structure/wrench matrix \(A^T\): maps cable tensions to platform wrench
- Shared geometric building blocks between kinematics and statics (unit cable directions, moment arms)

## 7. Cable models (`spyder-cables`)

Shared trait (conceptual):

```rust
trait CableModel {
    fn length(&self, a: Vec3, b: Vec3, ctx: &CableContext) -> Result<CableLength, ModelError>;
    // CableLength may carry geometric L and/or unstrained L0 + extras
}
```

| Model | When to use | Phase 1 |
|-------|-------------|---------|
| `Ideal` | Short/taut cables, maker default | Required |
| `Pulley` | Non-negligible pulley radius / wrap | Required |
| `Sag` (Irvine) | Long-span / heavy cable research | Required |

Implementation order inside Phase 1: **Ideal → Pulley → Sag**, same API throughout.

## 8. Statics (`spyder-statics`)

- Assemble \(A^T(pose)\) from cable directions (and \(b_i\) moment arms in platform mode)
- Solve \(A^T f + w = 0\) subject to \(f_{\min} \le f_i \le f_{\max}\)
- Solver ladder:
  1. Closed-form medium-force method (Pott) when applicable
  2. QP/LP fallback for bounded feasibility
- Config classification helper: IRPM / CRPM / RRPM from \(m\) and DOF
- `is_wrench_feasible(pose, wrench) -> bool` (+ optional witness tensions)
- Feeds `IkResult.tensions` when a wrench is supplied

Sag model + statics interaction: document and implement a clear kineto-static path (iterate tensions ↔ Irvine lengths to consistency, with iteration limits and convergence errors).

## 9. Actuation (`spyder-actuation`)

- `Winch`: constant drum radius in v1; optional variable spool diameter as later extension behind the same trait
- `Motor`: steps per revolution or encoder CPR, gear ratio, direction, optional velocity/acceleration limits
- Mapping: \(\Delta L \rightarrow \Delta\theta \rightarrow\) step counts (or radians)
- Synchronized move helper: given start/end poses (or length vectors), compute per-motor step counts and delays/rates so all winches complete together (segmented linear paths for straighter Cartesian motion — used by Phase 3; Phase 1 exposes the math helper)

## 10. API surface

### Rust

```rust
let robot = Robot::from_preset(Preset::Rect { width, depth, height, n: 4 }, Model::Ideal)?;
let ik = robot.ik(pose, IkOptions { wrench: Some(gravity), .. })?;
let fk = robot.fk(&ik.lengths, FkOptions::default())?;
let ok = robot.is_wrench_feasible(pose, gravity)?;
```

### Python

```python
from spyder import Robot

robot = Robot.from_preset("rect", width=10, depth=6, height=8, n=4, model="pulley")
# or Robot.from_anchors(anchors, platform_attachments=..., model="sag")
sol = robot.ik(pose, wrench=gravity)
pose2 = robot.fk(sol.lengths)
robot.check_feasible(pose, wrench=gravity)
```

### CLI

- `spyder ik --config venue.toml --xyz 1,2,3 [--rpy ...]`
- `spyder fk --config venue.toml --lengths ...`
- `spyder check --config venue.toml --xyz ...`

### Config format

TOML (primary) with JSON accepted at boundaries. Config includes anchors or preset, attachments, cable model, winch/motor params, tension bounds, default wrench.

## 11. Errors

All failures are typed and surfaced to Python as exceptions with stable codes:

| Condition | Behavior |
|-----------|----------|
| N < 3 or empty anchors | Config error |
| Invalid preset parameters | Config error |
| Degenerate geometry (coincident anchors, zero-length cable) | Geometry error |
| FK non-convergence | FkError with residual / iterations |
| No feasible tension in bounds | InfeasibleWrench |
| Rank-deficient structure matrix | SingularStructure |
| Sag iteration failure | ModelError |

No silent clamping of infeasible poses.

## 12. Testing strategy

- **Unit:** hand-computed ideal lengths for triangle/rectangle; pulley wrap fixtures; matrix rank for known CRPM/RRPM layouts
- **Property:** IK→FK round-trip within tolerance for random poses in a bounding box; invariance under rigid world transform
- **Golden configs:** `configs/` for 3-gon, 4-rect, 5-pentagon, irregular custom, platform-offset dolly
- **Numeric regression:** sag/tension paths checked against cited published numeric examples where reproducible
- **Python smoke:** load TOML → ik → fk → assert error < ε
- **CI:** `cargo test`, Python package import + CLI smoke

## 13. Phase 1 acceptance criteria

Phase 1 is complete when all of the following hold:

1. N ≥ 3 motors via preset **or** raw anchors  
2. Point-mass and platform modes work through one `Robot` API  
3. Ideal, pulley, and sag models selectable behind one API  
4. IK implemented for all three models  
5. Numerical FK plus at least one analytic/reduced FK fast path  
6. Tension solve + feasibility flag with bounds  
7. Winch/motor command mapping in `IkResult`  
8. Rust tests green; `spyder` Python package usable from CLI and notebooks  
9. Example configs and a short README documenting conventions and quickstart  

## 14. Later phases (hooks only)

**Phase 2 — `spyder-sim`:** visualize robot + cables, playback trajectories, sample wrench-feasible workspace volumes. Consumes `Robot`, `ik`, `fk`, `is_wrench_feasible`.

**Phase 3 — `spyder-runtime`:** backends for steppers / ODrive / similar; realtime loop calling actuation helpers; safety limits. Consumes `IkResult` motor commands and synchronized move helpers.

No Phase 2/3 implementation in Phase 1 beyond trait stability and documented extension points.

## 15. Decisions log

| Topic | Decision |
|-------|----------|
| Product scope | Full suite eventually; Phase 1 = core math + Python |
| Build order | Core IK → sim/viz → runtime |
| End-effector | Point-mass **and** 6-DOF platform (mode switch) |
| Languages | Rust core + Python (PyO3); C ABI later if needed |
| Cable models | Ideal + pulley + sag (most capable) |
| Layouts | Presets + raw anchors |
| IK outputs | Lengths + winch angles + motor cmds + tensions |
| FK | Numerical + analytic special cases |
| Architecture | Layered workspace (Approach 2) |
| License | MIT |

## 16. References (selected)

- Verhoeven / Ming–Higuchi — CDPR restraint classification (IRPM/CRPM/RRPM)
- Andreas Pott — *Cable-Driven Parallel Robots*; closed-form force distribution
- Merlet et al. — IK with sagging cables; Irvine catenary
- Gouttefarde / Nguyen — large-dimension CDPR, cable model simplifications
- Carricato — underconstrained geometrico-static problems
- CASPR (darwinlau/CASPR) — research analysis platform
- Maker: Marginally Clever Spidercam IK; EMS-TU-Ilmenau/SpiderRobot; MaltMover; samblazes pulley IK notes
- Commercial context: Spidercam / Skycam four-winch dolly architecture
