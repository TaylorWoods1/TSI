# Spyder Phase 1 Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Phase 1 of spyder — a parametric N-motor spider-cam IK/FK library with ideal/pulley/sag cable models, statics, actuation mapping, and Python bindings.

**Architecture:** Layered Rust Cargo workspace (`spyder-core`, `spyder-cables`, `spyder-statics`, `spyder-actuation`) with a `Robot` facade and PyO3 package `spyder-py`. Z-up meters, point-mass + platform modes, presets + raw anchors.

**Tech Stack:** Rust 2021 (nalgebra, thiserror, serde), PyO3/maturin, TOML configs, cargo test + pytest.

**Spec:** `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`

---

## File map

```
Cargo.toml                          # workspace
crates/spyder-core/src/
  lib.rs, error.rs, types.rs, pose.rs, anchor.rs
  preset.rs, robot.rs, ik.rs, fk.rs, jacobian.rs
crates/spyder-cables/src/
  lib.rs, model.rs, ideal.rs, pulley.rs, sag.rs
crates/spyder-statics/src/
  lib.rs, structure.rs, tension.rs, feasibility.rs
crates/spyder-actuation/src/
  lib.rs, winch.rs, motor.rs, mapping.rs
python/spyder/                      # PyO3 project (maturin)
  Cargo.toml, src/lib.rs, spyder/__init__.py
configs/*.toml
notebooks/01_ideal_rect.ipynb
README.md
```

---

### Task 1: Cargo workspace scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `crates/spyder-core/Cargo.toml`
- Create: `crates/spyder-core/src/lib.rs`
- Create: `crates/spyder-cables/Cargo.toml`
- Create: `crates/spyder-cables/src/lib.rs`
- Create: `crates/spyder-statics/Cargo.toml`
- Create: `crates/spyder-statics/src/lib.rs`
- Create: `crates/spyder-actuation/Cargo.toml`
- Create: `crates/spyder-actuation/src/lib.rs`
- Create: `LICENSE`

- [ ] **Step 1: Create workspace root `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = [
    "crates/spyder-core",
    "crates/spyder-cables",
    "crates/spyder-statics",
    "crates/spyder-actuation",
]
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["Spyder Contributors"]
[workspace.dependencies]
nalgebra = "0.33"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
approx = "0.5"
```

- [ ] **Step 2: Create each crate `Cargo.toml` + empty `lib.rs` with a doc comment**

`spyder-core` depends on `nalgebra`, `thiserror`, `serde`, `approx`, and path deps on the other three crates once they exist. For scaffold, only declare deps that exist; wire path deps in Task 2+.

Initial `spyder-core/Cargo.toml`:
```toml
[package]
name = "spyder-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
nalgebra = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
approx = { workspace = true }
```

Other crates similarly with `nalgebra` + `thiserror` only for now.

- [ ] **Step 3: Add MIT LICENSE file**

- [ ] **Step 4: Verify build**

Run: `cargo build`
Expected: success, four crates compile

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates LICENSE
git commit -m "chore: scaffold spyder Cargo workspace"
```

---

### Task 2: Core types — Vec3 alias, Pose, errors

**Files:**
- Create: `crates/spyder-core/src/error.rs`
- Create: `crates/spyder-core/src/types.rs`
- Create: `crates/spyder-core/src/pose.rs`
- Modify: `crates/spyder-core/src/lib.rs`
- Test: inline `#[cfg(test)]` in `pose.rs`

- [ ] **Step 1: Write failing Pose tests in `pose.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn identity_pose_leaves_point_unchanged() {
        let pose = Pose::identity();
        let p = Vec3::new(1.0, 2.0, 3.0);
        let out = pose.transform_point(&p);
        assert_relative_eq!(out.x, 1.0);
        assert_relative_eq!(out.y, 2.0);
        assert_relative_eq!(out.z, 3.0);
    }

    #[test]
    fn translation_only_pose() {
        let pose = Pose::from_position(Vec3::new(1.0, 0.0, 0.0));
        let out = pose.transform_point(&Vec3::new(0.0, 2.0, 0.0));
        assert_relative_eq!(out.x, 1.0);
        assert_relative_eq!(out.y, 2.0);
        assert_relative_eq!(out.z, 0.0);
    }
}
```

- [ ] **Step 2: Run test — expect fail (module missing)**

Run: `cargo test -p spyder-core`
Expected: compile fail or test fail

- [ ] **Step 3: Implement types**

```rust
// types.rs
pub type Vec3 = nalgebra::Vector3<f64>;
pub type Mat3 = nalgebra::Matrix3<f64>;
pub type UnitQuat = nalgebra::UnitQuaternion<f64>;

// pose.rs
#[derive(Clone, Debug, PartialEq)]
pub struct Pose {
    pub position: Vec3,
    pub orientation: UnitQuat,
}

impl Pose {
    pub fn identity() -> Self {
        Self {
            position: Vec3::zeros(),
            orientation: UnitQuat::identity(),
        }
    }
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            orientation: UnitQuat::identity(),
        }
    }
    pub fn transform_point(&self, body_point: &Vec3) -> Vec3 {
        self.position + self.orientation * body_point
    }
}

// error.rs
#[derive(Debug, thiserror::Error)]
pub enum SpyderError {
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("geometry error: {0}")]
    Geometry(String),
    #[error("FK did not converge (residual={residual}, iterations={iterations})")]
    FkNonConvergence { residual: f64, iterations: usize },
    #[error("wrench infeasible at pose")]
    InfeasibleWrench,
    #[error("singular structure matrix")]
    SingularStructure,
    #[error("cable model error: {0}")]
    Model(String),
}
pub type Result<T> = std::result::Result<T, SpyderError>;
```

Export from `lib.rs`.

- [ ] **Step 4: Run tests — expect pass**

Run: `cargo test -p spyder-core`

- [ ] **Step 5: Commit**

```bash
git add crates/spyder-core
git commit -m "feat(core): add Pose, Vec3 aliases, and error types"
```

---

### Task 3: Anchors, platform attachments, layout presets

**Files:**
- Create: `crates/spyder-core/src/anchor.rs`
- Create: `crates/spyder-core/src/preset.rs`
- Modify: `crates/spyder-core/src/lib.rs`

- [ ] **Step 1: Write failing preset tests**

```rust
#[test]
fn rect_preset_four_corners_at_height() {
    let anchors = rect(10.0, 6.0, 8.0).expect("ok");
    assert_eq!(anchors.len(), 4);
    // centered on origin in XY, z=height
    assert_relative_eq!(anchors[0].exit.z, 8.0);
    let xs: Vec<f64> = anchors.iter().map(|a| a.exit.x).collect();
    assert!(xs.contains(&5.0) || xs.iter().any(|x| (*x - 5.0).abs() < 1e-9));
}

#[test]
fn regular_polygon_n5() {
    let anchors = regular_polygon(5, 4.0, 7.0).expect("ok");
    assert_eq!(anchors.len(), 5);
}

#[test]
fn n_less_than_3_errors() {
    assert!(regular_polygon(2, 1.0, 1.0).is_err());
}
```

- [ ] **Step 2: Run — expect fail**

- [ ] **Step 3: Implement**

```rust
#[derive(Clone, Debug)]
pub struct Anchor {
    pub exit: Vec3,
    pub pulley_axis: Option<Vec3>, // unit axis; None => no pulley
    pub pulley_radius: f64,
}

#[derive(Clone, Debug)]
pub struct PlatformAttachment {
    pub body_point: Vec3,
}

pub fn rect(width: f64, depth: f64, height: f64) -> Result<Vec<Anchor>> {
    if width <= 0.0 || depth <= 0.0 {
        return Err(SpyderError::Config("width/depth must be > 0".into()));
    }
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let corners = [
        Vec3::new( hw,  hd, height),
        Vec3::new(-hw,  hd, height),
        Vec3::new(-hw, -hd, height),
        Vec3::new( hw, -hd, height),
    ];
    Ok(corners
        .into_iter()
        .map(|exit| Anchor {
            exit,
            pulley_axis: Some(Vec3::z()),
            pulley_radius: 0.0,
        })
        .collect())
}

pub fn regular_polygon(n: usize, radius: f64, height: f64) -> Result<Vec<Anchor>> {
    if n < 3 {
        return Err(SpyderError::Config("n must be >= 3".into()));
    }
    if radius <= 0.0 {
        return Err(SpyderError::Config("radius must be > 0".into()));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let th = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        out.push(Anchor {
            exit: Vec3::new(radius * th.cos(), radius * th.sin(), height),
            pulley_axis: Some(Vec3::z()),
            pulley_radius: 0.0,
        });
    }
    Ok(out)
}
```

- [ ] **Step 4: Tests pass**

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(core): anchors and rect/polygon layout presets"
```

---

### Task 4: Ideal cable model + ideal IK

**Files:**
- Create: `crates/spyder-cables/src/model.rs`
- Create: `crates/spyder-cables/src/ideal.rs`
- Modify: `crates/spyder-cables/src/lib.rs`
- Create: `crates/spyder-core/src/ik.rs`
- Modify: `crates/spyder-core/Cargo.toml` (depend on spyder-cables)
- Modify: `crates/spyder-core/src/lib.rs`

- [ ] **Step 1: Failing ideal length + IK tests**

In `spyder-cables`:
```rust
#[test]
fn ideal_length_is_euclidean() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(3.0, 4.0, 0.0);
    let m = Ideal;
    let len = m.length(&a, &b, &CableContext::default()).unwrap();
    assert_relative_eq!(len.geometric, 5.0);
}
```

In `spyder-core` ik tests — 2×2×1 rectangle, EE at center bottom-ish:
```rust
#[test]
fn ideal_ik_rect_center() {
    // anchors at (±1,±1,1), point-mass at (0,0,0)
    // each length = sqrt(1+1+1) = sqrt(3)
    let lengths = ideal_ik_point_mass(
        &[Vec3::new(1.,1.,1.), Vec3::new(-1.,1.,1.),
          Vec3::new(-1.,-1.,1.), Vec3::new(1.,-1.,1.)],
        &Vec3::new(0., 0., 0.),
    ).unwrap();
    for l in &lengths {
        assert_relative_eq!(*l, 3f64.sqrt(), epsilon = 1e-9);
    }
}
```

- [ ] **Step 2: Run — expect fail**

- [ ] **Step 3: Implement CableModel + Ideal + ik helpers**

```rust
pub struct CableContext { /* reserved for tension/sag */ }
impl Default for CableContext { fn default() -> Self { Self {} } }

pub struct CableLength {
    pub geometric: f64,
    pub unstrained: Option<f64>,
}

pub trait CableModel {
    fn length(&self, a: &Vec3, b: &Vec3, ctx: &CableContext) -> Result<CableLength, String>;
}

pub struct Ideal;
impl CableModel for Ideal {
    fn length(&self, a: &Vec3, b: &Vec3, _ctx: &CableContext) -> Result<CableLength, String> {
        let d = (b - a).norm();
        if d <= f64::EPSILON {
            return Err("zero-length cable".into());
        }
        Ok(CableLength { geometric: d, unstrained: None })
    }
}
```

IK: for each cable, `B = pose.transform_point(&attachment.body_point)`, then `model.length(anchor.exit, B)`.

- [ ] **Step 4: Tests pass**

- [ ] **Step 5: Commit**

```bash
git commit -am "feat: ideal cable model and inverse kinematics"
```

---

### Task 5: Numerical FK (LM) + round-trip property

**Files:**
- Create: `crates/spyder-core/src/fk.rs`
- Modify: `crates/spyder-core/Cargo.toml` (add `argmin` or implement simple LM manually to avoid heavy deps — **prefer hand-rolled Gauss-Newton** for point-mass 3DOF and 6DOF with finite-diff Jacobian to keep deps light)

- [ ] **Step 1: Failing round-trip test**

```rust
#[test]
fn ik_fk_round_trip_point_mass() {
    let anchors = rect(4.0, 4.0, 3.0).unwrap();
    let p = Vec3::new(0.3, -0.2, 1.0);
    let lengths = ik_point_mass_ideal(&anchors, &p).unwrap();
    let recovered = fk_point_mass_numeric(&anchors, &lengths, Vec3::new(0., 0., 1.5)).unwrap();
    assert_relative_eq!(recovered.x, p.x, epsilon = 1e-6);
    assert_relative_eq!(recovered.y, p.y, epsilon = 1e-6);
    assert_relative_eq!(recovered.z, p.z, epsilon = 1e-6);
}
```

- [ ] **Step 2: Run — fail**

- [ ] **Step 3: Implement Gauss-Newton FK for point-mass**

Minimize `sum (||p - a_i|| - L_i)^2`. Jacobian row i: `(p - a_i)/||p - a_i||`. Iterate until residual < 1e-10 or max 50 iters; return `SpyderError::FkNonConvergence` on failure.

- [ ] **Step 4: Tests pass**

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(core): numerical point-mass forward kinematics"
```

---

### Task 6: Analytic FK fast paths (3-cable + 4-rect)

**Files:**
- Create: `crates/spyder-core/src/fk_analytic.rs`
- Modify: `crates/spyder-core/src/fk.rs` (dispatch)

- [ ] **Step 1: Tests for trilateration and rect dispatch**

```rust
#[test]
fn three_cable_analytic_fk() {
    // known tetrahedron geometry
}

#[test]
fn fk_dispatch_uses_analytic_for_rect4() {
    let result = fk_auto(...).unwrap();
    assert_eq!(result.method, FkMethod::AnalyticRect4);
}
```

- [ ] **Step 2: Implement trilateration (two intersections → pick higher-Z or closer to seed)**

- [ ] **Step 3: Implement rect-4 reduced solver or call numeric with analytic seed**

- [ ] **Step 4: Tests pass + commit**

```bash
git commit -am "feat(core): analytic FK fast paths for 3-cable and rect-4"
```

---

### Task 7: Robot facade (point-mass + platform modes)

**Files:**
- Create: `crates/spyder-core/src/robot.rs`
- Create: `crates/spyder-core/src/jacobian.rs`

- [ ] **Step 1: Test platform mode offsets attachment points**

```rust
#[test]
fn platform_mode_changes_lengths_vs_point_mass() {
    // same pose, nonzero b_i => different L
}
```

- [ ] **Step 2: Implement `Robot` with `from_preset`, `from_anchors`, `ik`, `fk`, `point_mass` flag**

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(core): Robot facade with point-mass and platform modes"
```

---

### Task 8: Pulley cable model

**Files:**
- Create: `crates/spyder-cables/src/pulley.rs`

- [ ] **Step 1: Test wrap arc increases length vs ideal for nonzero radius**

```rust
#[test]
fn pulley_length_exceeds_euclidean_when_radius_positive() {
    // axis = Z, radius = 0.05, horizontal span
}
```

- [ ] **Step 2: Implement swivel-pulley tangent + arc (Sam Blazes / Pott geometry)**

- [ ] **Step 3: Wire into Robot model enum; tests pass; commit**

```bash
git commit -am "feat(cables): swivel-pulley length model"
```

---

### Task 9: Structure matrix + tension distribution

**Files:**
- Implement `crates/spyder-statics/src/{structure,tension,feasibility}.rs`
- Depend from spyder-core

- [ ] **Step 1: Test 4-cable point-mass at center under gravity has positive tensions**

- [ ] **Step 2: Implement Aᵀ assembly + Pott closed-form + simple QP/clamping fallback**

Use `nalgebra` SVD for pseudoinverse. Bounds `fmin`, `fmax`.

- [ ] **Step 3: `is_wrench_feasible`; commit**

```bash
git commit -am "feat(statics): structure matrix and tension distribution"
```

---

### Task 10: Irvine sag model (kineto-static)

**Files:**
- Create: `crates/spyder-cables/src/sag.rs`

- [ ] **Step 1: Test sag unstrained length > geometric chord for heavy cable under tension**

- [ ] **Step 2: Implement Irvine equations; iterate with tension solve for consistency**

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(cables): Irvine sag kineto-static model"
```

---

### Task 11: Actuation mapping

**Files:**
- Implement `crates/spyder-actuation/src/{winch,motor,mapping}.rs`

- [ ] **Step 1: Test ΔL maps to expected steps for known drum radius and steps/rev**

```rust
#[test]
fn length_to_steps() {
    // radius 0.05 m, 200 steps/rev, ΔL = 2π*0.05 => 200 steps
}
```

- [ ] **Step 2: Implement; synchronized timing helper; commit**

```bash
git commit -am "feat(actuation): winch and motor command mapping"
```

---

### Task 12: Python bindings + CLI

**Files:**
- Create: `python/` maturin project exposing `Robot`, `ik`, `fk`
- Create: `configs/rect_4.toml`
- Update: `README.md`

- [ ] **Step 1: maturin develop; pytest round-trip**

- [ ] **Step 2: CLI `spyder ik --config ... --xyz ...`**

- [ ] **Step 3: README quickstart; commit**

```bash
git commit -am "feat(python): PyO3 bindings, CLI, example config, README"
```

---

### Task 13: Golden configs + property tests + Phase 1 gate

**Files:**
- Create: `configs/{triangle_3,rect_4,pentagon_5,irregular,platform_offset}.toml`
- Expand tests for transform invariance and N-motor polygon IK

- [ ] **Step 1: Add golden + property tests**

- [ ] **Step 2: Run full `cargo test` and Python smoke**

- [ ] **Step 3: Verify Phase 1 acceptance checklist from spec §13**

- [ ] **Step 4: Final commit**

```bash
git commit -am "test: golden layouts and Phase 1 acceptance coverage"
```

---

## Milestone notes

Ship order inside this plan is strict: **scaffold → types → presets → ideal IK → numeric FK → analytic FK → Robot → pulley → statics → sag → actuation → Python → goldens**. Do not start pulley before ideal round-trips pass. Do not start sag before statics tension solve works.

## Self-review

- Spec §13 acceptance mapped to Tasks 3–13
- No TBD steps; concrete commands and code
- Types consistent: `Pose`, `Anchor`, `Robot`, `CableModel`, `IkResult` naming aligned with spec
