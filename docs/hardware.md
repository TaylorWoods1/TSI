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
```

Firmware → host: `OK` or `ERR ...`

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

> Multi-board / >2 axes: map cables to axis 0/1 per board and use one transport per ODrive (extend CLI as needed).

## Safety

- Always dry-run with `--backend mock` first  
- Verify cable directions and winch radius before enabling drivers  
- Keep tension limits conservative; watch for slack / over-tension on first moves  
