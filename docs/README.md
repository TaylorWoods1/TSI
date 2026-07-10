# Spyder documentation

Index for the Spyder cable-robot (spider-cam) project.

## Getting started

| Doc | Audience | Contents |
|-----|----------|----------|
| [../README.md](../README.md) | Everyone | Quick starts: Rust CLI, Python, GUI, hardware |
| [gui.md](gui.md) | GUI users & contributors | Local GUI, dev workflow, API smoke tests |
| [hardware.md](hardware.md) | Operators | Motor backends, stepper protocol, calibration |
| [cli.md](cli.md) | CLI users | `spyder` subcommands and examples |
| [config-schema.md](config-schema.md) | Config authors | Venue TOML fields and examples |

## Architecture

| Doc | Contents |
|-----|----------|
| [architecture.md](architecture.md) | Crate graph, data flow, conventions |

## Package READMEs

| Path | Contents |
|------|----------|
| [../web/README.md](../web/README.md) | React / Vite frontend |
| [../python/README.md](../python/README.md) | PyO3 bindings and pytest |

## Planning archive

Historical specs and implementation plans (may lag `main` — check README for shipped status):

- `superpowers/specs/2026-07-10-spyder-ik-design.md` — core IK/FK design
- `superpowers/specs/2026-07-10-spyder-gui-design.md` — GUI design (MVP shipped; see MVP table in spec)
- `superpowers/plans/` — phase-1 and GUI implementation plans

## Testing

```bash
cargo test --workspace          # all Rust crates + integration tests
cargo test -p spyder-gui        # GUI API route tests
cd web && npm test              # Vitest (API client)
cd python && pytest tests/      # Python bindings (after maturin develop)
```

See [gui.md](gui.md) for GUI-specific testing and two-process dev setup.
