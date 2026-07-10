# Spyder CLI

Binary: `spyder` (`cargo run -p spyder-cli -- …`)

All kinematics commands take a **venue TOML** as the first argument. See [config-schema.md](config-schema.md).

## Commands

### Kinematics

```bash
spyder ik <config.toml> <x,y,z>
spyder fk <config.toml> <l1,l2,...> [seed_x,y,z]
spyder workspace <config.toml> [out_prefix]
```

`workspace` writes `out_prefix.csv`, `.json`, and `.html` (default prefix `workspace`).

### Visualization

```bash
spyder scene <config.toml> <x,y,z> [out.html]
  [--to x,y,z] [--segments N] [--workspace]
```

Static pose HTML, or animated trajectory with optional workspace point cloud.

### Calibration

```bash
spyder calibrate <config.toml> <x,y,z> [out.json]
spyder field-cal <x,y,z;x,y,z;...> <home_x,y,z> [out.toml] [--drum R] [--steps N]
spyder venue-from-cal <cal.json> [out.toml]
```

### Playback

```bash
spyder play <config.toml> <x0,y0,z0> <x1,y1,z1> [segments]
  [--backend mock|stepper|odrive|multiboard]
  [--device PATH|host:port] [--baud N]
  [--closed-loop] [--realtime]
  [--cal cal.json] [--axis-map map.json]
```

| Backend | `--device` example |
|---------|-------------------|
| `mock` | (none) — dry-run |
| `stepper` | `/dev/ttyUSB0` or `127.0.0.1:9002` |
| `odrive` | `/dev/ttyACM0` |
| `multiboard` | requires `--axis-map` JSON |

### Utilities

```bash
spyder axis-map-example [out.json]
```

Generates a dual-ODrive axis map template. **Create the file before referencing it:**

```bash
cargo run -p spyder-cli -- axis-map-example configs/axis_map_dual_odrive.json
```

## Library

`spyder-cli` exposes `robot_from_toml` for tests and future tooling:

```rust
use spyder_cli::robot_from_toml;
```

Implementation: `crates/spyder-cli/src/toml.rs`.

## Examples

```bash
mkdir -p artifacts

cargo run -p spyder-cli -- ik configs/rect_4.toml 0,0,2
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 --backend mock
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene.html \
  --to 0.5,0,1.5 --segments 12 --workspace
```

Hardware workflows: [hardware.md](hardware.md).
