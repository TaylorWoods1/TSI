# Hardware backends

Spyder talks to motors through `MotorBackend` implementations over a line `Transport` (serial or TCP).

## Backends

| Backend | Flag | Device | Notes |
|---------|------|--------|-------|
| Mock | `--backend mock` | — | Dry-run, no I/O |
| Stepper | `--backend stepper` | `/dev/ttyUSB0` or `127.0.0.1:9002` | Multi-axis line protocol + Arduino firmware |
| ODrive | `--backend odrive` | serial path | ASCII `q` position commands in turns |

## Stepper protocol

Host → firmware (newline-terminated):

```
M <n> <steps0> <delay0_us> <steps1> <delay1_us> ...
H
P
E
```

| Cmd | Meaning |
|-----|---------|
| `M` | Multi-axis step burst |
| `H` | Hardware home / zero positions |
| `P` | Report current step positions |
| `E` | E-stop acknowledge / halt |

Firmware → host: `OK`, `OK <steps…>` (for `P`), or `ERR ...`

Flash `firmware/spyder_stepper/spyder_stepper.ino` to an Arduino/ESP32. Default pins: STEP 2–5, DIR 6–9, 115200 baud.

### Local simulator (no hardware)

```bash
cargo run -p spyder-stepper-sim -- 9002
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend stepper --device 127.0.0.1:9002
```

### Real serial

```bash
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend stepper --device /dev/ttyUSB0 --baud 115200
```

## ODrive

Uses the [ASCII protocol](https://docs.odriverobotics.com/v/latest/manual/ascii-protocol.html):

1. `w axisN.requested_state 8` — closed loop  
2. `q <axis> <turns> <vel_lim>` — position setpoints  

Step deltas from the Player are converted to turns via `steps_per_rev` (default 200).

```bash
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 0.5,0,2 5 \
  --backend odrive --device /dev/ttyACM0 --baud 115200
```

## Calibration + home

Capture home pose lengths and measured anchors:

```bash
cargo run -p spyder-cli -- calibrate configs/rect_4.toml 0,0,1.5 artifacts/cal.json
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,1.5 0.2,0,1.5 20 \
  --backend mock --cal artifacts/cal.json --closed-loop
```

`Player` applies calibration, homes software zeros, and can correct pose from encoder/step feedback (`P` / ODrive `f`) when `--closed-loop` is set.

### Field calibration → venue TOML

Measure anchor XYZ in the venue frame, then generate a loadable config:

```bash
cargo run -p spyder-cli -- field-cal \
  "5,3,8;-5,3,8;-5,-3,8;5,-3,8" 0,0,1.5 artifacts/venue.toml

# Or convert an existing calibration JSON
cargo run -p spyder-cli -- calibrate configs/rect_4.toml 0,0,1.5 artifacts/cal.json
cargo run -p spyder-cli -- venue-from-cal artifacts/cal.json artifacts/venue.toml
```

## Safety

`SafetyLimits` in the Player enforce soft workspace bounds, max speed, cable length range, and step-burst size. E-stop is available via `MotorBackend::estop` (firmware `E`).

- Always dry-run with `--backend mock` first  
- Verify cable directions and winch radius before enabling drivers  
- Keep tension limits conservative; watch for slack / over-tension on first moves  
- Use `--closed-loop` only after feedback reads look sane  

## 3D scene

```bash
# Static pose
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene.html

# Animated trajectory with play/scrub + optional workspace cloud
cargo run -p spyder-cli -- scene configs/rect_4.toml 0,0,1.5 artifacts/scene_anim.html \
  --to 0.5,0,1.5 --segments 12 --workspace
```

## Multi-board axis map

```bash
cargo run -p spyder-cli -- axis-map-example configs/axis_map_dual_odrive.json

# Dry-run fan-out across mapped devices (mock boards)
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,1.5 0.2,0,1.5 8 \
  --backend mock --axis-map configs/axis_map_dual_odrive.json

# Live: open one stepper transport per device in the map
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,1.5 0.2,0,1.5 8 \
  --backend multiboard --axis-map configs/axis_map_dual_odrive.json --realtime
```

`Player::with_realtime(true)` sleeps per segment for wall-clock playback.

## GUI API (dev)

Start the local GUI server (default `http://127.0.0.1:7700`):

```bash
cargo run -p spyder-gui
```

Smoke the JSON API:

```bash
curl -s localhost:7700/health
curl -s -X POST localhost:7700/venue/from_preset \
  -H 'Content-Type: application/json' \
  -d '{"kind":"rect","width":10,"depth":6,"height":8,"point_mass":true}'
curl -s -X POST localhost:7700/ik \
  -H 'Content-Type: application/json' \
  -d '{"xyz":[0,0,2]}'
curl -s -X POST localhost:7700/workspace \
  -H 'Content-Type: application/json' \
  -d '{"min":[-2,-2,0.5],"max":[2,2,4],"nx":5,"ny":5,"nz":4,"mg":9.81,"f_min":0.5,"f_max":500}'
```

See `docs/superpowers/specs/2026-07-10-spyder-gui-design.md` for the full route table.
