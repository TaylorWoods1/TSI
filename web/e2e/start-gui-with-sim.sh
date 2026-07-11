#!/usr/bin/env bash
# Start stepper TCP sim in background, then spyder-gui (for Playwright E2E).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

cargo run -p spyder-stepper-sim -- 9002 &
STEPPER_PID=$!
trap 'kill "$STEPPER_PID" 2>/dev/null || true' EXIT

for _ in $(seq 1 60); do
  if (echo >/dev/tcp/127.0.0.1/9002) >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

exec cargo run -p spyder-gui
