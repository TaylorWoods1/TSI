# Venue configuration (TOML)

Venue files describe anchor geometry and platform mode. They are used by:

- `spyder-cli` (`ik`, `fk`, `play`, `scene`, …)
- `spyder-gui` (`POST /venue/load`, save/export)
- Python (load via CLI or manual construction)

Parsers: `crates/spyder-cli/src/toml.rs`, `crates/spyder-gui/src/toml_venue.rs`

## Supported fields

### Preset layout

```toml
preset = "rect"      # or "polygon"
width = 10.0         # rect only (meters)
depth = 6.0
height = 8.0
n = 6                # polygon only — cable count
radius = 5.0         # polygon — circumradius (meters)
point_mass = true    # true = cables meet at dolly origin
```

### Explicit anchors

```toml
point_mass = true

[[anchors]]
x = 5.0
y = 3.0
z = 8.0

[[attachments]]   # optional; platform mode
x = 0.2
y = 0.0
z = 0.0
```

When `[[anchors]]` entries are present, preset fields are ignored.

### Home pose

```toml
[home]
x = 0.0
y = 0.0
z = 2.0
```

Used by GUI run session and field-cal output. CLI `play` uses pose arguments separately unless calibration JSON overrides lengths.

## Example files (`configs/`)

| File | Description |
|------|-------------|
| `rect_4.toml` | Default 4-corner rectangle, point-mass |
| `triangle_3.toml` | 3-cable CRPM layout |
| `pentagon_5.toml` | 5-cable polygon preset |
| `irregular.toml` | Custom `[[anchors]]` positions |
| `platform_offset.toml` | Rigid platform with attachment offsets |
| `hardware_stepper.toml` | Rect preset + **documentation-only** actuation notes |

## Unsupported / ignored sections

The following are **not** parsed by venue loaders today:

```toml
[actuation]          # NOT read by CLI/GUI venue parser
drum_radius_m = 0.05
steps_per_rev = 200
gear_ratio = 1.0
baud = 115200
```

Pass hardware parameters via CLI flags instead:

```bash
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0,2 8 \
  --backend stepper --device /dev/ttyUSB0 --baud 115200
```

Calibration JSON (`spyder calibrate …`) stores `drum_radius_m` and `steps_per_rev` for playback.

## Comments

Lines starting with `#` are ignored. Inline comments after values are supported:

```toml
width = 10.0  # meters
```

## Field calibration output

`spyder field-cal` and `venue-from-cal` emit TOML with explicit `[[anchors]]`, `[home]`, and optional `drum_radius_m` / `steps_per_rev` in the header (via `spyder-runtime::venue_toml_from_anchors`).
