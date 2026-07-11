# Spyder desktop shell (Tauri)

Native desktop wrapper around the same **Design → Simulate → Run** web GUI. The shell spawns a bundled `spyder-gui` sidecar (Axum API + static `web/dist`) and opens a webview to `http://127.0.0.1:7700`.

## Prerequisites

- Rust **1.88** (pinned in repo root `rust-toolchain.toml`)
- Node.js 20+
- Linux build deps:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libudev-dev libappindicator3-dev pkg-config
```

## Quick start (development)

```bash
cd apps/spyder-tauri
npm install
npm run dev
```

`tauri dev` runs `scripts/prepare-sidecar.sh debug` first (builds `spyder-gui` + `web/dist`, stages the sidecar binary), then launches the desktop window.

Override backend binary path (dev only):

```bash
export SPYDER_GUI_BIN=/path/to/spyder-gui
npm run dev
```

## Release build (Linux .deb)

From the repo root or `apps/spyder-tauri`:

```bash
cd apps/spyder-tauri
npm install
npm run build          # .deb bundle
# npm run build:all    # deb + rpm + appimage (appimage needs extra host tooling)
```

Installer output:

```
apps/spyder-tauri/src-tauri/target/release/bundle/deb/Spyder_0.1.0_amd64.deb
```

Install locally:

```bash
sudo dpkg -i apps/spyder-tauri/src-tauri/target/release/bundle/deb/Spyder_0.1.0_amd64.deb
spyder-tauri
```

## Layout

```
apps/spyder-tauri/
  package.json              # @tauri-apps/cli scripts
  scripts/prepare-sidecar.sh  # build spyder-gui + web/dist, stage sidecar
  src-tauri/
    Cargo.toml              # Rust shell + plugins
    tauri.conf.json         # bundle config (sidecar + web/dist resources)
    binaries/               # staged spyder-gui-<target-triple> (gitignored)
    icons/                  # app icons
    src/lib.rs              # sidecar spawn, single-instance, tray
    capabilities/           # shell sidecar permissions
```

## How it works

1. **Single instance** — `tauri-plugin-single-instance` focuses the existing window if a second launch is attempted.
2. **Sidecar** — `spyder-gui` is bundled via `externalBin` and spawned with `tauri-plugin-shell`. `SPYDER_WEB_DIST` points at bundled `web/dist` resources.
3. **Port wait** — polls TCP **7700** until the API is up.
4. **Tray** — system tray menu with **Quit**; graceful sidecar shutdown on exit.
5. **Dev fallback** — if the sidecar is unavailable, falls back to `target/debug|release/spyder-gui` from the workspace.

## API contract

No Tauri-specific API. The webview uses the same JSON routes as the browser — see [gui-configurator.md](./gui-configurator.md).

## CI

The `tauri` job in `.github/workflows/ci.yml` runs `cargo check`, builds the `.deb`, and uploads it as a CI artifact.

## See also

- [gui.md](./gui.md) — browser workflow
- [gui-configurator.md](./gui-configurator.md) — feature reference
