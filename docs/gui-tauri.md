# Spyder desktop shell (Tauri) — planned

The web GUI (`web/` + `spyder-gui` on port 7700) is the primary interface today. A future **Tauri** wrapper would package the same React SPA as a native desktop app with bundled backend startup.

## Planned layout

```
apps/spyder-tauri/
  src-tauri/     # Rust shell: spawn spyder-gui, open webview
  package.json   # Vite dev proxy to :7700
```

## Responsibilities (not yet implemented)

1. **Process management** — start `spyder-gui` as a child process; restart on crash.
2. **Single instance** — one window per machine; optional tray icon.
3. **Serial permissions** — macOS/Linux udev hints; Windows COM port picker.
4. **Deep links** — `spyder://venue/load?path=...` for venue TOML files.
5. **Offline assets** — serve `web/dist` from Tauri instead of separate browser tab.

## Dev workflow (target)

```bash
# Terminal 1 — API + static (unchanged)
cargo run -p spyder-gui

# Terminal 2 — Tauri dev (future)
cd apps/spyder-tauri && npm run tauri dev
```

## API contract

No Tauri-specific API is required. The desktop shell talks to the same JSON routes documented in [gui-configurator.md](./gui-configurator.md).

## Status

**Stub only.** Track implementation in a dedicated issue when desktop distribution is prioritized.
