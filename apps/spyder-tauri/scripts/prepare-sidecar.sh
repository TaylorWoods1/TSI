#!/usr/bin/env bash
# Build spyder-gui + web/dist and stage the sidecar binary for Tauri bundling.
set -euo pipefail

PROFILE="${1:-release}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
TAURI_DIR="$ROOT/apps/spyder-tauri/src-tauri"
TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"

cd "$ROOT"
if [[ "$PROFILE" == "release" ]]; then
  cargo build -p spyder-gui --release
  BIN="$ROOT/target/release/spyder-gui"
else
  cargo build -p spyder-gui
  BIN="$ROOT/target/debug/spyder-gui"
fi

cd "$ROOT/web"
npm ci
npm run build

mkdir -p "$TAURI_DIR/binaries"
cp "$BIN" "$TAURI_DIR/binaries/spyder-gui-$TRIPLE"
chmod +x "$TAURI_DIR/binaries/spyder-gui-$TRIPLE"
echo "Staged sidecar: binaries/spyder-gui-$TRIPLE"
