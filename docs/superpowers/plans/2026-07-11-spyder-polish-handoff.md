# Spyder — Polish & Hardening Handoff

**Date:** 2026-07-11  
**Baseline:** `main` @ `7509ff8` (Rust **1.88.0**, full configurator shipped)  
**Prior status:** [2026-07-10-spyder-next.md](2026-07-10-spyder-next.md)  
**Stocktake:** 2026-07-11 top-down review (core done; polish/CI/packaging remain)

---

## Context snapshot

| Item | Value |
|------|-------|
| Repo | `TaylorWoods1/TSI` (branded **Spyder** internally) |
| Toolchain | `rust-toolchain.toml` → **1.88.0** (Tauri lockfile needs ≥1.86) |
| GUI | Axum `spyder-gui` `:7700` + Vite/React/R3F `web/` |
| Desktop | `apps/spyder-tauri/` — dev shell only (not in workspace, no CI) |
| Tests | **131** Rust workspace, **24** `spyder-gui`, **6** Playwright E2E, **9** Python |
| Shipped | Design / Simulate / Run, all hardware backends, calibration→venue TOML, motor mapping |

### Known gotchas (do not rediscover)

1. **Vite dev proxy** — `web/vite.config.ts` proxies `/health`, `/venue`, `/ik`, … `/run` but **not `/calibration`**. Field calibration on `:5173` fails unless requests hit `:7700` directly.
2. **Tauri outside workspace** — `apps/spyder-tauri/src-tauri` is separate; `cargo test --workspace` does not build it.
3. **`web/dist` not committed** — `cargo run -p spyder-gui` warns and serves API-only until `cd web && npm run build`.
4. **Stale docs** — `docs/gui-configurator.md` §Desktop still says Tauri is a "planned stub"; `README.md` says "107+ tests" (actual: **131**).
5. **`[actuation]` in venue TOML** — documentation-only; not parsed by loaders ([config-schema.md](../../config-schema.md)).
6. **Simulate play ≠ Run play** — Simulate animates dolly client-side; Run drives motors via `/run/*`.

### Explicit non-goals (unless user asks)

- Sag cable model rewrite (Phase-1 approximation in `spyder-cables`)
- Analytic FK expansion beyond current 3-cable / rect-4 cases
- PyPI publish / maturin release pipeline
- Full Tauri installer (`.deb`/`.msi`) — Phase C scaffolds only
- GitHub repo rename `TSI` → `spyder` (admin action)

---

## Phase A — Quick wins (one session)

Low risk, high signal. Do these first.

### A.1 Fix Vite calibration proxy

- [ ] **Task:** Add `/calibration` proxy entry in `web/vite.config.ts` → `http://127.0.0.1:7700`
- [ ] **Verify:** `cargo run -p spyder-gui` + `cd web && npm run dev`; Design tab → Field calibration → Capture succeeds on `:5173`
- [ ] **Files:** `web/vite.config.ts`

### A.2 Documentation hygiene

- [ ] **Task:** `docs/gui-configurator.md` — replace §"Desktop shell (planned)" with link to shipped [gui-tauri.md](../../gui-tauri.md) scaffold
- [ ] **Task:** `README.md` — update test count to **131** (or "130+")
- [ ] **Task:** `docs/superpowers/plans/2026-07-10-spyder-full-configurator.md` — add banner at top: `ARCHIVED — completed on main; see spyder-next.md`
- [ ] **Optional:** `docs/superpowers/plans/2026-07-10-spyder-gui.md` — note Rust 1.88 supersedes 1.83 mention (archive only)
- [ ] **Files:** `docs/gui-configurator.md`, `README.md`, planning archive

### A.3 Clippy in CI (optional but recommended)

- [ ] **Task:** Add step to `.github/workflows/ci.yml` `rust` job: `cargo clippy --workspace -- -D warnings` (or `-D warnings` only on changed crates if too noisy initially)
- [ ] **Verify:** `cargo clippy --workspace` passes locally on 1.88
- [ ] **Files:** `.github/workflows/ci.yml`

**Phase A acceptance**

```bash
cargo test --workspace
cargo test -p spyder-gui
cd web && npm ci && npm test && npm run build && npm run e2e
```

---

## Phase B — Quality gates (1–2 sessions)

### B.1 Tauri compile in CI

- [ ] **Task:** Add CI job `tauri` (or step in `gui` job):
  - Install `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libudev-dev`, `pkg-config`
  - `cd apps/spyder-tauri/src-tauri && cargo check`
- [ ] **Verify:** Same command passes locally
- [ ] **Files:** `.github/workflows/ci.yml`
- [ ] **Note:** Do not add full `tauri build` until Phase C bundling is designed

### B.2 E2E: calibration smoke

- [ ] **Task:** Playwright test in `web/e2e/configurator.spec.ts`:
  - Design tab → expand Field calibration → Capture (with default rect venue)
  - Assert success toast or calibration state visible
  - Click Export venue TOML → verify download or API `GET /calibration/venue_toml` via page context
- [ ] **Files:** `web/e2e/configurator.spec.ts`

### B.3 E2E: TOML round-trip

- [ ] **Task:** Playwright test:
  - Download venue TOML from Design
  - Re-upload via load control
  - Assert anchor count / classify unchanged
- [ ] **Files:** `web/e2e/configurator.spec.ts`

### B.4 Rust API test gaps (optional)

- [ ] **Task:** Add `spyder-gui` route tests for uncovered paths:
  - `GET /venue`, `POST /calibration/capture`, `POST /run/disconnect`, `POST /scene/export`
- [ ] **Files:** `crates/spyder-gui/src/api.rs`, `cal_svc.rs`

**Phase B acceptance**

```bash
# All Phase A checks plus:
cd apps/spyder-tauri/src-tauri && cargo check
cd web && npm run e2e   # includes new calibration + TOML tests
```

---

## Phase C — Desktop packaging (larger)

Only start after Phase A+B are green. User should confirm they want distributable builds.

### C.1 Bundle spyder-gui binary

- [ ] **Task:** Tauri `bundle` resources or `externalBin` for `spyder-gui`
- [ ] **Task:** `beforeBuildCommand` / `beforeDevCommand` in `tauri.conf.json` or documented script to build backend + `web/dist`
- [ ] **Verify:** Fresh clone → single command produces runnable desktop app
- [ ] **Files:** `apps/spyder-tauri/src-tauri/tauri.conf.json`, `apps/spyder-tauri/package.json`, `docs/gui-tauri.md`

### C.2 Single-instance lock

- [ ] **Task:** Prevent multiple Tauri launches from fighting for `:7700` (file lock, port check, or Tauri single-instance plugin)
- [ ] **Verify:** Second launch shows error or focuses existing window
- [ ] **Files:** `apps/spyder-tauri/src-tauri/src/lib.rs`

### C.3 Tray icon & lifecycle (optional)

- [ ] **Task:** System tray with Quit; graceful `spyder-gui` child shutdown
- [ ] **Files:** `apps/spyder-tauri/src-tauri/`, `docs/gui-tauri.md`

### C.4 Enable Tauri bundling

- [ ] **Task:** `bundle.active: true`; generate/store icons; document Linux build deps
- [ ] **CI:** Optional `tauri build` on `ubuntu-latest` artifact upload
- [ ] **Files:** `tauri.conf.json`, `.github/workflows/ci.yml`

**Phase C acceptance**

- Desktop app starts on clean machine with bundled binary
- Only one instance owns `:7700`
- `docs/gui-tauri.md` documents release build path

---

## Phase D — Optional expansion

Pick individually; no required order.

### D.1 TCP sim E2E in CI

- [ ] Spawn `cargo run -p spyder-stepper-sim -- 9002` in Playwright `webServer` or dedicated CI job
- [ ] Run tab → stepper backend → `127.0.0.1:9002` → connect + home (mock-free)
- [ ] **Files:** `web/e2e/`, `.github/workflows/ci.yml`

### D.2 UX polish

- [ ] TransformControls rotation gizmo on Design dolly (platform mode)
- [ ] Waypoint drag-reorder in Simulate tab
- [ ] Run tab: editable `duration_s` for waypoint playback (currently hardcoded `2.0` in `RunPage.tsx`)
- [ ] Safety limits editor (currently read-only from `/run/status`)

### D.3 Python expansion

- [ ] Venue TOML loader on `spyder.Robot`
- [ ] Optional `Player` / mock backend bindings
- [ ] **Files:** `python/src/`, `python/tests/`

### D.4 Firmware & ops

- [ ] `firmware/README.md` — flash instructions, pin map, `SPYDER_MAX_AXES`
- [ ] CI: `arduino-cli compile` on `firmware/spyder_stepper/spyder_stepper.ino`
- [ ] GitHub rename `TSI` → `spyder` (admin UI, update clone URLs in docs)

---

## Verification commands (all phases)

```bash
# Rust
cargo test --workspace
cargo test -p spyder-gui
cargo clippy --workspace -- -D warnings    # after A.3

# Web
cd web && npm ci && npm test && npm run build && npm run e2e

# Python (CI parity)
cd python && python -m venv .venv && source .venv/bin/activate
pip install maturin==1.4.0 pytest && maturin develop --release && pytest tests/ -q

# Tauri (after B.1)
cd apps/spyder-tauri/src-tauri && cargo check
```

**Linux deps:** `libudev-dev pkg-config` (workspace); Tauri adds `libwebkit2gtk-4.1-dev libgtk-3-dev`.

---

## Paste prompts for new chats

### Prompt — Phase A only (recommended first handoff)

```
Repo: TaylorWoods1/TSI (Spyder CDPR). Branch from main (latest).

Execute Phase A from:
  docs/superpowers/plans/2026-07-11-spyder-polish-handoff.md

Tasks:
  A.1 — Add /calibration to web/vite.config.ts proxy
  A.2 — Doc hygiene (gui-configurator Tauri section, README test count, archive banner on full-configurator plan)
  A.3 — Add cargo clippy --workspace to CI (fix any warnings)

RULES:
- Minimal diffs; match existing conventions.
- Do NOT touch sag model, PyPI, or Tauri bundling.
- Branch: cursor/polish-phase-a-ee08
- Commit + push; open draft PR to main.

Verify before done:
  cargo test --workspace && cargo test -p spyder-gui
  cd web && npm ci && npm test && npm run build && npm run e2e
```

### Prompt — Phase B (after A merged)

```
Repo: TaylorWoods1/TSI. Branch from main.

Execute Phase B from:
  docs/superpowers/plans/2026-07-11-spyder-polish-handoff.md

Tasks:
  B.1 — Tauri cargo check in CI (WebKitGTK deps)
  B.2 — Playwright calibration smoke E2E
  B.3 — Playwright TOML round-trip E2E
  B.4 — (optional) spyder-gui API tests for uncovered routes

Branch: cursor/polish-phase-b-ee08
Verify: Phase A acceptance + apps/spyder-tauri/src-tauri cargo check + e2e
```

### Prompt — Phase C (user must confirm packaging goal)

```
Repo: TaylorWoods1/TSI. Branch from main.

Execute Phase C from:
  docs/superpowers/plans/2026-07-11-spyder-polish-handoff.md

Goal: distributable Tauri desktop app with bundled spyder-gui + web/dist.

Tasks: C.1 bundle binary, C.2 single-instance, C.3 tray (optional), C.4 enable bundling.

Branch: cursor/tauri-packaging-ee08
Update docs/gui-tauri.md with release build instructions.
```

### Prompt — Phase D item (pick one)

```
Repo: TaylorWoods1/TSI. Branch from main.

From docs/superpowers/plans/2026-07-11-spyder-polish-handoff.md, implement:
  [ ] D.1 TCP sim E2E in CI
  [ ] D.2 UX: waypoint duration control on Run tab
  [ ] D.3 Python venue TOML loader
  [ ] D.4 firmware README + arduino-cli CI

Specify which D.* items in your first reply. Minimal scope per item.
```

---

## Definition of done (full polish track)

| Phase | Done when |
|-------|-----------|
| **A** | Calibration works on Vite dev `:5173`; docs accurate; clippy green in CI |
| **B** | Tauri `cargo check` in CI; ≥2 new E2E tests pass |
| **C** | Installable desktop build; single-instance; docs updated |
| **D** | Per-item acceptance in handoff |

When Phase A+B are merged, update [2026-07-10-spyder-next.md](2026-07-10-spyder-next.md) shipped table and check off completed items in this file.

---

## File reference

| Path | Relevance |
|------|-----------|
| `web/vite.config.ts` | A.1 proxy fix |
| `web/e2e/configurator.spec.ts` | B.2, B.3 E2E |
| `.github/workflows/ci.yml` | A.3, B.1, C.4, D.4 |
| `docs/gui-configurator.md` | A.2 stale Tauri section |
| `README.md` | A.2 test count |
| `apps/spyder-tauri/src-tauri/` | B.1, Phase C |
| `crates/spyder-gui/src/api.rs` | B.4 route tests |
| `web/src/pages/RunPage.tsx` | D.2 duration_s hardcode |
