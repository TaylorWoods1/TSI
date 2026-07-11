# Spyder Tauri shell

Desktop wrapper for the Spyder web GUI. See [docs/gui-tauri.md](../../docs/gui-tauri.md).

```bash
cargo build -p spyder-gui
cd ../../web && npm ci && npm run build && cd ../apps/spyder-tauri
npm install
npm run tauri dev
```
