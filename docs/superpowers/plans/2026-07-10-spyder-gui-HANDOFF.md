# HANDOFF — Spyder GUI (new chat)

> **SUPERSEDED (2026-07-10):** GUI MVP is implemented on `main`. Use [docs/gui.md](../../gui.md) and [README.md](../../../README.md) for current setup. This file is kept for historical context.

## Paste this as the new chat’s first message

```
Implement the Spyder GUI.

Read and follow:
- docs/superpowers/specs/2026-07-10-spyder-gui-design.md
- docs/superpowers/plans/2026-07-10-spyder-gui.md

Branch: create `cursor/spyder-gui-55c0` from `main`.
Skill: subagent-driven-development (or executing-plans).
Start at Task 1. Do not reimplement IK/FK — use existing spyder-* crates.
Ship milestone GUI-1 (Axum API + tests) first; commit/push; then continue GUI-2/3.
```

## Context snapshot

- Repo: still GitHub `TaylorWoods1/TSI` (rename to spyder blocked)
- `main` tip when planned: `9a189e8` — core, sim, runtime, field-cal, 6DOF FK all merged
- No GUI code yet — only Plotly HTML exports + CLI
- Plan branch for these docs: `cursor/spyder-gui-plan-55c0` (merge docs to main before implementing)

## Success = acceptance criteria in the design spec §10
