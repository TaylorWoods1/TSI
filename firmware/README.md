# Spyder stepper firmware

Arduino-compatible multi-axis stepper firmware for the Spyder line protocol (`spyder-runtime::StepperBackend`).

Source: `spyder_stepper/spyder_stepper.ino`

## Protocol

Newline-terminated commands; replies with `OK` or `ERR <msg>`.

| Command | Description |
|---------|-------------|
| `M <n> <steps0> <delay0_us> ...` | Interleaved move on `n` axes |
| `H` | Hardware home — zero position counters |
| `P` | Report positions: `P s0 s1 ...` |
| `E` | E-stop / halt |

On boot the firmware sends `OK spyder-stepper`.

## Pin map (default 4 axes)

| Axis | STEP | DIR |
|------|------|-----|
| 0 | 2 | 6 |
| 1 | 3 | 7 |
| 2 | 4 | 8 |
| 3 | 5 | 9 |

Optional enable pin: `EN_PIN` (default `-1`, disabled).

Edit `STEP_PINS`, `DIR_PINS`, and `NUM_AXES` in the sketch for your board.

## `SPYDER_MAX_AXES`

Compile-time cap on axis array size (default **8**). Set when building:

```bash
arduino-cli compile --build-property "compiler.cpp.extra_flags=-DSPYDER_MAX_AXES=8" ...
```

`NUM_AXES` in the sketch is the active axis count (≤ `SPYDER_MAX_AXES`).

## Flash (Arduino UNO / AVR)

```bash
# Install toolchain once
arduino-cli core update-index
arduino-cli core install arduino:avr

# Compile
arduino-cli compile --fqbn arduino:avr:uno firmware/spyder_stepper

# Upload (replace port)
arduino-cli upload -p /dev/ttyUSB0 --fqbn arduino:avr:uno firmware/spyder_stepper
```

## Local test without hardware

```bash
cargo run -p spyder-stepper-sim -- 9002
cargo run -p spyder-cli -- play configs/rect_4.toml 0,0,2 1,0.5,2 8 \
  --backend stepper --device 127.0.0.1:9002
```

Or use the GUI Run tab with backend **stepper** and device `127.0.0.1:9002`.

## See also

- [docs/hardware.md](../docs/hardware.md) — backends and calibration
- [docs/cli.md](../docs/cli.md) — `play` command
