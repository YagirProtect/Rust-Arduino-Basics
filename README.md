# Rust Arduino Basics (Arduino Uno / ATmega328P)

Small **Rust + `arduino-hal`** playground for **Arduino Uno** (**ATmega328P @ 16 MHz**).

Repo layout:

- **Firmware**: `Arduino/` — `#![no_std]` examples + a tiny project-local “mini-std” layer (timer + serial logging).
- **PC tools**: `Tools/` — helper utilities (e.g. a serial streamer for raw 8‑bit audio). citeturn2view0turn2view1

> Target board: **Arduino Uno / ATmega328P @ 16 MHz**. citeturn11view5

---

## Quick start

```bash
cd Arduino
# edit Ravedude.toml -> set your serial port (COM5, /dev/ttyACM0, ...)
cargo build
cargo run
```

Notes:

- `Arduino/.cargo/config.toml` sets `target = "avr-none"` and uses **ravedude** as the runner. citeturn11view4
- This project uses `build-std = ["core"]` for the AVR target. citeturn11view4
- `GlobalTimer` reconfigures **Timer0** (like Arduino core does). If you rely on Arduino-core `millis()/delay()`, don’t. citeturn7view7
- Use `wrapping_sub` for time deltas (the millis counter is `u32` and wraps). citeturn7view7turn9view11

---

## Prerequisites

1) **Rust nightly** pinned in `Arduino/rust-toolchain.toml`. citeturn2view0

2) AVR tooling:
- `avr-gcc`
- `avr-libc`
- `avrdude`

3) Flasher/runner:
- `ravedude` citeturn11view4turn11view5

Canonical setup docs: https://github.com/Rahix/avr-hal#readme citeturn1view0

---

## Wiring cheatsheet

### I²C bus (shared by LCD + sensors)
On **Uno/Nano (ATmega328P)**:
- `SDA` → `A4`
- `SCL` → `A5`

All I²C devices connect **in parallel** to the same SDA/SCL and share **GND**.  
Each device must have its own **I²C address**.

### LCD1602 + I²C backpack (PCF8574)
- `VCC` → `5V`
- `GND` → `GND`
- `SDA` → `A4`
- `SCL` → `A5`

Common addresses: `0x27`, `0x3F`  
If backlight is on but no text: adjust the **contrast potentiometer**.

---

## Firmware modules (expand)

The firmware crate exports modules under `Arduino/src/modules/` and helpers under `Arduino/src/std/`. citeturn11view0turn11view2

### Tiny “mini-std” layer

<details>
  <summary><b>GlobalTimer</b> — Timer0 millis (CTC + Compare A ISR)</summary>

**What:** global millisecond counter backed by **Timer0** interrupt. citeturn7view7  
**Why:** schedule periodic tasks without blocking `delay()`.

**Notes**
- ATmega328P @ 16 MHz, prescaler 64, `OCR0A=249` → ~1 ms tick. citeturn7view7
- Requires interrupts enabled after init. citeturn7view7turn8view10
- `u32` wraps naturally → use `wrapping_sub`. citeturn7view7

**Example**
```rust
use crate::std::global_timer::GlobalTimer;
use crate::std::std::enable_interrupts;

let timer = GlobalTimer::new(&dp.TC0);
enable_interrupts();

let mut last = timer.millis();
loop {
    let now = timer.millis();
    if now.wrapping_sub(last) >= 200 {
        last = now;
        // do something every 200 ms
    }
}
```
</details>

<details>
  <summary><b>IoUno</b> — UART logger (heapless buffer + newline send)</summary>

**What:** serial logger without `std`, using a `heapless::String<64>` scratch buffer and sending it over **USART0**. citeturn9view13turn9view14

**Example**
```rust
use core::fmt::Write;
use crate::std::io::IoUno;

let mut io = IoUno::new(dp.USART0, pins.d0, pins.d1, 115200);

let x = 123;
let y = 456;

writeln!(io.str(), "x={}, y={}", x, y).ok();
io.log(); // sends the buffer + newline
```
</details>

<details>
  <summary><b>Math helpers</b> — small no_std utilities (internal)</summary>

**What:** `inverse_lerp`, `lerp`, `normalize` (float helpers). citeturn10view7

**Important:** in the current repo `std/math.rs` is a private submodule (`mod math;`), so it is intended for internal use / future re-exporting. citeturn11view2
</details>

---

### Drivers (“modules”)

<details>
  <summary><b>LCD1602 (HD44780) over I²C (PCF8574)</b> — blocking + queued (“async”)</summary>

**What:** character LCD driver through an I²C backpack (PCF8574). citeturn7view0turn11view1  
**Why “slow”:** HD44780 has slow commands (`clear/home` ~1.5ms), and with PCF8574 each character becomes multiple I²C writes (send high nibble + low nibble, each latched by `E`). citeturn7view0turn8view3

**API in this repo**
- `ScreenLCD1602::get_line()` — clears and returns a `heapless::String<64>` scratch buffer. citeturn8view0turn7view0
- `ScreenLCD1602::print(&mut i2c)` — prints the current buffer, interpreting `\n` as the second row. citeturn7view0turn8view3
- `EMode::Strait` — blocking (bytes sent immediately). citeturn7view0turn8view3
- `EMode::Async` — enqueues commands/data; you must call `update(now_ms, &mut i2c)` in the main loop. citeturn7view3turn8view3

**Blocking example (Strait)**
```rust
use core::fmt::Write;
use crate::modules::screen_lcd1602::screen_lcd1602::{EMode, ScreenLCD1602};

let mut lcd = ScreenLCD1602::new(0x27, &mut i2c, EMode::Strait);
write!(lcd.get_line(), "Hello!\nWorld").unwrap();
lcd.print(&mut i2c);
```

**Queued example (Async)**
```rust
use core::fmt::Write;
use crate::modules::screen_lcd1602::screen_lcd1602::{EMode, ScreenLCD1602};

let mut lcd = ScreenLCD1602::new(0x27, &mut i2c, EMode::Async);

write!(lcd.get_line(), "pressed!!!").unwrap();
lcd.print(&mut i2c); // enqueues clear + cursor + bytes

loop {
    let now = timer.millis();
    lcd.update(now, &mut i2c); // executes queued ops (one per call)
}
```

**Tips**
- Right now `print()` always clears the display first. If you call it often, it will feel slow. citeturn8view3
- If backlight is on but no text: adjust contrast pot + confirm address (`0x27` / `0x3F`).

</details>

<details>
  <summary><b>Joystick HW-504</b> — analog X/Y + optional SW button</summary>

**What:** reads joystick axes via ADC and optional SW button via pull-up. citeturn7view4turn9view10

**Wiring (typical)**
- `VRx` → `A0`, `VRy` → `A1`, `SW` → e.g. `D7` (use pull-up)

**Example**
```rust
use crate::modules::joystick_hw504::JoystickHW504;

let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

let x = pins.a0.into_analog_input(&mut adc);
let y = pins.a1.into_analog_input(&mut adc);
let sw = pins.d7.into_pull_up_input();

let mut js = JoystickHW504::new(Some(x), Some(y), Some(sw), 8);

loop {
    let now = timer.millis();
    js.update(now, &mut adc);
    let _x = js.x_raw();
    let _y = js.y_raw();
    let _pressed = js.button_pressed();
}
```
</details>

<details>
  <summary><b>Light sensor (LDR)</b> — analog read + optional power gating</summary>

**What:** reads an LDR divider via ADC; can optionally power the sensor from a GPIO. citeturn9view0turn9view3

**Constructor in this repo**
- `LightSensorResistor::new(power_pin: Option<OutputPin>, output_pin: AnalogPin, read_rate_ms: u32)` citeturn9view1

**Example (with power gating)**
```rust
use crate::modules::light_sensor_resistor::LightSensorResistor;

let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

let power = pins.d7.into_output();
let analog = pins.a0.into_analog_input(&mut adc);

let mut ldr = LightSensorResistor::new(Some(power), analog, 50);
ldr.set_power(true);

loop {
    let now = timer.millis();
    ldr.update(now, &mut adc);
    if ldr.is_read() {
        let raw = ldr.last_data();
        let pct = ldr.percent();
        let _ = (raw, pct);
    }
}
```

**Note:** this module’s `MAX_INPUT_VALUE` is currently `512` (project-specific calibration). citeturn7view5turn10view2
</details>

<details>
  <summary><b>BFS Water Sensor</b> — analog + power-gating (anti-corrosion)</summary>

**What:** powers the probe only during read: `HIGH → ADC → LOW`. citeturn7view6turn10view0  
**Why:** reduces electrolysis/corrosion and noise on exposed-trace sensors. citeturn7view6

**Example**
```rust
use crate::modules::water_sensor_bfs::WaterSensorBFS;

let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

let power = pins.d7.into_output();
let analog = pins.a0.into_analog_input(&mut adc);

let mut water = WaterSensorBFS::new(power, analog, 500);

loop {
    let now = timer.millis();
    water.update(now, &mut adc);

    if water.is_read() {
        let raw = water.last_data();
        let pct = water.percent(); // f32
        let _ = (raw, pct);
    }
}
```
</details>

<details>
  <summary><b>Analog temperature sensor (LM25/LM35-style)</b> — ADC to °C / °F without floats</summary>

**What:** reads analog temperature and converts using integer math (assumes 10 mV/°C). citeturn6view1turn10view6  
**API:** `to_celsius() -> (int, frac)` and `to_fahrenheit() -> (int, frac)`. citeturn10view6

**Example**
```rust
use crate::modules::temperature_sensor_lm25::TemperatureSensorLM25;

let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
let analog = pins.a0.into_analog_input(&mut adc);

let mut t = TemperatureSensorLM25::new(analog, 250);

loop {
    let now = timer.millis();
    t.update(now, &mut adc);

    if t.is_read() {
        let (c_int, c_frac) = t.to_celsius();
        let (f_int, f_frac) = t.to_fahrenheit();
        let _ = (c_int, c_frac, f_int, f_frac);
    }
}
```
</details>

---

## Tools (expand)

<details>
  <summary><b>Serial RAW audio streamer</b> — Tools/MP3_SERIAL_STREAM</summary>

Streams a `*.raw` file (unsigned **u8 mono PCM**) to a serial port at real-time speed. citeturn12view0turn11view7

**Run**
```bash
cd Tools/MP3_SERIAL_STREAM
cargo run --release -- COM5 bad_apple.raw 250000 8000 256
```

Args:

1) `PORT`  – `COM5`, `/dev/ttyACM0`, ...
2) `FILE`  – path to `*.raw`
3) `BAUD`  – default `250000`
4) `RATE`  – samples/sec, default `8000`
5) `CHUNK` – bytes per write, default `256` citeturn11view7
</details>

---

## Troubleshooting

<details>
  <summary><b>I²C “no response” / device not detected</b></summary>

- Check **GND** (loose ground is #1).
- Verify SDA/SCL are on **A4/A5** (Uno/Nano).
- Try the other LCD address (`0x27` ↔ `0x3F`).
- Keep wires short; if unstable, reduce I²C speed (100 kHz is already used in the firmware examples). citeturn10view11
</details>

<details>
  <summary><b>LCD backlight on but no text</b></summary>

- Adjust **contrast** potentiometer.
- Confirm correct I²C address.
- Some PCF8574 backpacks use a different pin mapping (rare).
</details>

---

## Cross-platform note (Linux/macOS case-sensitive filesystems)

In `Arduino/src/modules/`, module declarations use **snake_case** names (e.g. `pub mod joystick_hw504;`) but some filenames in the repo use mixed case (e.g. `joystick_HW504.rs`). On Windows this usually works; on case-sensitive filesystems it can fail to compile. citeturn4view0turn11view0

If you want this repo to build everywhere, rename the files to match the module names exactly, e.g.:

- `joystick_HW504.rs` → `joystick_hw504.rs`
- `water_sensor_BFS.rs` → `water_sensor_bfs.rs`
- `temperature_sensor_LM25.rs` → `temperature_sensor_lm25.rs` citeturn4view0turn11view0

---

## Licenses

Dual-licensed under either of:

- Apache License 2.0 — `LICENSE-APACHE`
- MIT License — `LICENSE-MIT` citeturn1view0

---

## Contributing

PRs/issues welcome. By default, contributions are dual-licensed under Apache-2.0 OR MIT.
