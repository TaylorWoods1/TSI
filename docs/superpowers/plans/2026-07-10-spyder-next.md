# Spyder — What's Next

**Date:** 2026-07-10 (updated)  
**Spec:** `docs/superpowers/specs/2026-07-10-spyder-ik-design.md`

## Shipped

| Area | Status |
|------|--------|
| Phase 1–4 core / sim / runtime / harden | Done |
| Animated scene + multi-board + realtime | Done |
| Platform 6DOF FK + field-cal | Done |
| **GUI MVP** (Axum + React, mock Run) | Done — see [docs/gui.md](../../gui.md) |
| Expanded test suite (Rust, Python, Vitest) | Done |

## Next (planned)

| Priority | Item |
|----------|------|
| GUI-4 | Playwright E2E, polish, optional Tauri shell |
| GUI Run | Stepper/ODrive backends in GUI (CLI already supports) |
| GUI Design | Platform mode, cable model picker, calibration export |
| Docs | Keep `docs/` index current as features land |

Specs: `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`  
Plan: `docs/superpowers/plans/2026-07-10-spyder-gui.md`

## Remaining ops

1. GitHub rename `TSI` → `spyder` (admin UI), if desired  
2. Optional: commit generated `configs/axis_map_dual_odrive.json` or document generator-only workflow (done in [docs/cli.md](../../cli.md))
