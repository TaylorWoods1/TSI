# Spyder web UI

Vite + React + React Three Fiber frontend for the local Spyder GUI.

## Prerequisites

- Node 20+
- Running API: `cargo run -p spyder-gui` (or bundled build — see below)

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run dev` | Vite dev server on `:5173` with API proxy to `:7700` |
| `npm run build` | Production bundle → `dist/` (required for Axum static serve) |
| `npm test` | Vitest unit tests |
| `npm run preview` | Preview production build |

## Project layout

```
src/
  App.tsx              Tab shell (Design | Simulate | Run)
  context.tsx          Shared venue, dolly pose, trajectory config
  api/client.ts        HTTP client for :7700 API
  pages/
    DesignPage.tsx     Venue edit, TOML load/save, draggable anchors
    SimulatePage.tsx   Trajectory play, workspace overlay
    RunPage.tsx        Mock connect, play, E-stop
  scene/
    RobotScene.tsx     R3F canvas — anchors, cables, dolly (Z-up)
  styles.css           Spyder chrome tokens
```

## API client

`src/api/client.ts` wraps the subset of routes the UI uses. The server exposes additional endpoints (`/fk`, `/jacobian`, `/feasible`) documented in [../docs/gui.md](../docs/gui.md).

Base URL logic:

- **Dev** (`npm run dev` on port 5173): relative paths, proxied by Vite
- **Production** (served from `:7700`): `http://127.0.0.1:7700`

## 3D conventions

- World frame: **Z-up** (`THREE.Object3D.DEFAULT_UP.set(0, 0, 1)`)
- Anchors: teal spheres; dolly: orange octahedron; cables: gray lines
- `RobotScene` accepts `SceneData`: `{ anchors, dolly, lengths, workspace? }`

## Testing

```bash
npm test
```

Covers API client error parsing and success paths (`src/api/client.test.ts`).

## Production bundle

The Rust server embeds/serves this directory after build:

```bash
npm ci && npm run build
cargo run -p spyder-gui
```

Without `dist/`, only the JSON API is available.
