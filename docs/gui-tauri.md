# Spyder desktop shell (Tauri)

Native desktop wrapper around the same **Design → Simulate → Run** web GUI. The shell spawns `spyder-gui` (Axum API + static `web/dist`) and opens a webview to `http://127.0.0.1:7700`.

## Prerequisites

- Rust **1.85+** (pinned in repo root `rust-toolchain.toml`; shared by `spyder-gui` and Tauri shell)
- Node.js 20+
- Linux: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libudev-dev`, `pkg-config`
- Built backend: `cargo build -p spyder-gui`
- Built UI: `cd web && npm ci && npm run build`

## Quick start

```bash
# 1. Build backend + web assets (once)
cargo build -p spyder-gui
cd web && npm ci && npm run build && cd ..

# 2. Install Tauri CLI (once)
cd apps/spyder-tauri && npm install

# 3. Run desktop app (spawns spyder-gui automatically)
npm run tauri dev
```

Override backend binary path:

```bash
export SPYDER_GUI_BIN=/path/to/spyder-gui
npm run tauri dev
```

## Layout

```
apps/spyder-tauri/
  package.json          # @tauri-apps/cli scripts
  src-tauri/
    Cargo.toml          # Rust shell
    tauri.conf.json     # window + remote URL
    src/lib.rs          # spawn spyder-gui, wait for :7700
    capabilities/       # remote localhost access
```

## How it works

1. On launch, the shell resolves `spyder-gui` (`SPYDER_GUI_BIN`, then `target/debug|release/spyder-gui`, then `$PATH`).
2. Polls TCP port **7700** until the API is up.
3. Opens a Tauri window loading `http://127.0.0.1:7700`.
4. On exit, kills the child `spyder-gui` process.

## API contract

No Tauri-specific API. The webview uses the same JSON routes as the browser — see [gui-configurator.md](./gui-configurator.md).

## Not yet implemented

- Bundling `spyder-gui` inside the `.deb` / `.msi` installer
- System tray icon
- Single-instance lock
- `spyder://` deep links for venue TOML files

## See also

- [gui.md](./gui.md) — browser workflow
- [gui-configurator.md](./gui-configurator.md) — feature reference
