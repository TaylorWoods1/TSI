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

See **[2026-07-11-spyder-polish-handoff.md](2026-07-11-spyder-polish-handoff.md)** for phased tasks (A–D), checklists, and copy-paste prompts for new chats.

| Priority | Item |
|----------|------|
| **Phase A** | Vite `/calibration` proxy, doc hygiene, clippy CI |
| **Phase B** | Tauri `cargo check` in CI, calibration + TOML E2E |
| **Phase C** | Bundle `spyder-gui`, single-instance, Tauri release |
| **Phase D** | TCP sim CI, UX polish, Python TOML, firmware README |

Specs: `docs/superpowers/specs/2026-07-10-spyder-gui-design.md`  
Archive: `docs/superpowers/plans/2026-07-10-spyder-full-configurator.md` (completed)
