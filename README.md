# Arduino Uno (ATmega328P) playground in Rust

This repository is a small **Rust + `arduino-hal`** playground for **Arduino Uno**.
It contains:

- **Firmware** (`Arduino/`) – `#![no_std]` examples + a tiny “mini-std” layer (timer + serial logging).
- **PC tools** (`Tools/`) – helper utilities (e.g. a serial streamer for raw 8‑bit audio).

> Target board: **Arduino Uno / ATmega328P @ 16 MHz**.

---

## What’s inside

### Firmware (`Arduino/`)

**Tiny “std” layer**

- `std/global_timer.rs` – `millis()` via **Timer0 CTC + ISR** (1 ms tick).
- `std/io.rs` – UART logger (`uFmt` + `heapless::String`).
- `std/math.rs` – a couple of helpers (`inverse_lerp`, `lerp`, `normalize`).

**Modules (drivers-ish)**

- `modules/joystick.rs` – analog joystick (X/Y) + optional SW button.
- `modules/light_sensor.rs` – analog light sensor with optional power-gating pin.
- `modules/water_sensor.rs` – BFS water sensor (analog) with **power-gating** to reduce corrosion.

### Tools (`Tools/`)

- `Tools/MP3_SERIAL_STREAM/` – streams a `*.raw` **u8 mono PCM** file to a COM port at real-time speed.

---

## Prerequisites

### Firmware toolchain (Windows/macOS/Linux)

1) **Rust nightly** pinned in `Arduino/rust-toolchain.toml`.

2) AVR tooling:

- `avr-gcc`
- `avr-libc`
- `avrdude`

3) Flasher/runner:

- [`ravedude`](https://crates.io/crates/ravedude)

The canonical setup steps are documented in the `avr-hal` README:
https://github.com/Rahix/avr-hal#readme

---

## Build & flash firmware

From the repo root:

```bash
cd Arduino
```

1) Edit `Arduino/Ravedude.toml` and set the right `port` (e.g. `COM5`).

2) Build:

```bash
cargo build
```

3) Flash + open serial console:

```bash
cargo run
```

Notes:

- `.cargo/config.toml` sets `target = "avr-none"` and uses `ravedude` as the runner.
- `GlobalTimer` reconfigures **Timer0**. If you rely on Arduino-core `millis()/delay()`, don’t.
- Time comparisons should use `wrapping_sub` (counter is `u32` and wraps ~ every 49.7 days).

---

## Wiring cheatsheet

### Analog joystick (HW-504 / similar)

- `VCC` → `5V`
- `GND` → `GND`
- `VRx` → `A0`
- `VRy` → `A1`
- `SW`  → e.g. `D7` (**INPUT_PULLUP**)

### Light sensor (LDR module / analog out)

- `AO` → `A0`
- Optional: power-gate from `D7` (module VCC controlled by a GPIO)

### BFS water sensor (analog)

- `S` / `AO` → `A0`
- `GND` → `GND`
- `VCC` → controlled by `D7` (power-gating pin)

Power-gating is recommended for “exposed trace” sensors to reduce electrolysis/corrosion.

---

## Usage examples

### Joystick

`modules/joystick.rs` contains a commented example; the idea is:

```rust
// X = A0, Y = A1, SW = D7 (pull-up)
let analog0 = pins.a0.into_analog_input(&mut adc);
let analog1 = pins.a1.into_analog_input(&mut adc);
let button  = pins.d7.into_pull_up_input();

let mut joystick = Joystick::new(Some(analog0), Some(analog1), Some(button), 8);

loop {
    let now = timer.millis();
    joystick.update(now, &mut adc);
    // joystick.x_raw(), joystick.y_raw(), joystick.button(), joystick.button_pressed()
}
```

### Water sensor (BFS)

```rust
let mut power = pins.d7.into_output();
let analog0 = pins.a0.into_analog_input(&mut adc);
let mut water = WaterSensorBFS::new(power, analog0, 500);

loop {
    let now = timer.millis();
    water.update(now, &mut adc);
    if water.is_read() {
        // water.last_data()  (0..1023 on Uno ADC)
    }
}
```

> ADC note: Uno’s ADC is **10-bit** → codes are **0..1023** (1024 levels).

---

## Tool: Serial raw audio streamer

Located at `Tools/MP3_SERIAL_STREAM/`.

This tool streams a `*.raw` file (unsigned 8‑bit mono PCM) to a serial port at the specified sample rate.

### Build & run

```bash
cd Tools/MP3_SERIAL_STREAM
cargo +stable run --release -- COM5 bad_apple.raw 250000 8000 256
```

Args:

1) `PORT`  – `COM5`, `/dev/ttyACM0`, ...
2) `FILE`  – path to `*.raw`
3) `BAUD`  – default `250000`
4) `RATE`  – samples/sec, default `8000`
5) `CHUNK` – bytes per write, default `256`

### Convert audio to RAW

If you have `ffmpeg` installed:

```bash
ffmpeg -i input.mp3 -ac 1 -ar 8000 -f u8 output.raw
```

> To actually *play* streamed audio you’ll need firmware that reads serial into a ring buffer and outputs PWM (or a DAC).

---

## Licenses

Firmware is dual-licensed under either of:

- Apache License 2.0 — `Arduino/LICENSE-APACHE`
- MIT License — `Arduino/LICENSE-MIT`

---

## Contributing

PRs/issues welcome. By default, contributions are dual-licensed under Apache-2.0 OR MIT.
