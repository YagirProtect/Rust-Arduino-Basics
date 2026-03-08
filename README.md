# Rust Arduino Basics (Arduino Uno / ATmega328P)

Small **Rust + `arduino-hal`** playground for **Arduino Uno** (ATmega328P @ 16 MHz).

- **Firmware** (`Arduino/`) — `#![no_std]` examples + a tiny “mini-std” layer (timer + serial logging).
- **PC tools** (`Tools/`) — helper utilities (e.g. a serial streamer for raw 8‑bit audio).

> Target board: **Arduino Uno / ATmega328P @ 16 MHz**.

---

## Quick start

```bash
cd Arduino
# edit Ravedude.toml -> set your serial port (COM5, /dev/ttyACM0, ...)
cargo build
cargo run
```

Notes:
- `.cargo/config.toml` sets `target = "avr-none"` and uses `ravedude` as the runner.
- `GlobalTimer` reconfigures **Timer0**. If you rely on Arduino-core `millis()/delay()`, don’t.
- Use `wrapping_sub` for time deltas (u32 wraps ~ every 49.7 days).

---

## Prerequisites

1) **Rust nightly** pinned in `Arduino/rust-toolchain.toml`.

2) AVR tooling:
- `avr-gcc`
- `avr-libc`
- `avrdude`

3) Flasher/runner:
- `ravedude`

Canonical setup docs: https://github.com/Rahix/avr-hal#readme

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

### Tiny “mini-std” layer

<details>
  <summary><b>GlobalTimer</b> — Timer0 millis (CTC + Compare A ISR)</summary>

**What:** global millisecond counter backed by **Timer0** interrupt.  
**Why:** schedule periodic tasks without blocking `delay()`.

**Notes**
- ATmega328P @ 16 MHz, prescaler 64, `OCR0A=249` → ~1 ms tick
- Requires interrupts enabled after init
- `u32` wraps naturally → use `wrapping_sub`

**Example**
  ```rust
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
  <summary><b>IO logger</b> — UART logging (ufmt + heapless)</summary>

**What:** tiny serial logger without `std`, using `ufmt` macros and a `heapless::String` buffer.

**Example**
  ```rust
  use ufmt::uwriteln;

  io.clear();
  uwriteln!(io.str(), "x={}, y={}", x, y).ok();
  io.log(); // sends buffer over UART
  ```
</details>

<details>
  <summary><b>Math helpers</b> — small no_std utilities</summary>

**What:** small helpers like `inverse_lerp`, `lerp`, `normalize` to keep drivers clean.

**Example**
  ```rust
  let t = inverse_lerp(0.0, 1023.0, raw as f32); // 0..1
  let v = lerp(-1.0, 1.0, t);                     // -1..1
  ```
</details>

---

### Drivers (“modules”)

<details>
  <summary><b>LCD1602 (HD44780) over I²C (PCF8574)</b> — blocking + queued (“async”)</summary>

**What:** character LCD driver through an I²C backpack (PCF8574).  
**Why “slow”:** HD44780 has slow commands (`clear/home` ~1.5ms), and with PCF8574 each char becomes multiple I²C writes.

**Tips**
- Don’t `clear()` every frame: update only what changed.
- If text missing: adjust **contrast** (pot) and verify address (`0x27` / `0x3F`).
- Async mode = enqueue ops + call `update(now, &mut i2c)` in the loop.

**Blocking example**
  ```rust
  let mut lcd = ScreenLCD1602::new(0x27, &mut i2c, EMode::Strait);
  lcd.set_cursor(&mut i2c, 0, 0);
  lcd.print_str(&mut i2c, "Hello!");
  ```

**Queued example**
  ```rust
  use core::fmt::Write;

  let mut lcd = ScreenLCD1602::new(0x27, &mut i2c, EMode::Async);

  lcd.clear_line();
  write!(lcd.get_line(), "pressed!!!").unwrap();
  lcd.print(&mut i2c); // enqueue clear + cursor + bytes

  loop {
      let now = timer.millis();
      lcd.update(now, &mut i2c);
  }
  ```
</details>

<details>
  <summary><b>Joystick HW-504</b> — analog X/Y + SW button</summary>

**What:** reads joystick axes via ADC and optional SW button via `INPUT_PULLUP`.

**Wiring**
- `VRx` → `A0`, `VRy` → `A1`, `SW` → e.g. `D7` (pull-up)

**Example**
  ```rust
  let x = pins.a0.into_analog_input(&mut adc);
  let y = pins.a1.into_analog_input(&mut adc);
  let sw = pins.d7.into_pull_up_input();

  let mut js = Joystick::new(Some(x), Some(y), Some(sw), 8);

  loop {
      let now = timer.millis();
      js.update(now, &mut adc);
      // js.x_raw(), js.y_raw(), js.button(), js.button_pressed()
  }
  ```
</details>

<details>
  <summary><b>Light sensor (LDR)</b> — analog read + optional power gating</summary>

**What:** analog light sensor module (LDR) read via ADC.  
**Optional:** power the module from a GPIO to reduce idle draw / for experiments.

**Example**
  ```rust
  let analog = pins.a0.into_analog_input(&mut adc);
  let mut sensor = LightSensorResistor::new(analog, /*read_rate_ms*/ 50);

  loop {
      let now = timer.millis();
      sensor.update(now, &mut adc);
      let raw = sensor.last_data(); // 0..1023
  }
  ```
</details>

<details>
  <summary><b>BFS Water Sensor</b> — analog + power-gating (anti-corrosion)</summary>

**What:** reads BFS water sensor (exposed traces) via ADC.  
**Power-gating:** recommended to reduce electrolysis/corrosion (turn VCC on only during read).

**Example**
  ```rust
  let mut power = pins.d7.into_output();
  let analog = pins.a0.into_analog_input(&mut adc);

  let mut water = WaterSensorBFS::new(power, analog, 500);

  loop {
      let now = timer.millis();
      water.update(now, &mut adc);
      if water.is_read() {
          let raw = water.last_data(); // 0..1023
      }
  }
  ```
</details>

<details>
  <summary><b>Analog temperature sensor (LM25/LM35-style)</b> — ADC to °C</summary>

**What:** simple analog temperature sensor using ADC (10-bit on Uno).

**Example**
  ```rust
  let analog = pins.a0.into_analog_input(&mut adc);
  let mut t = TemperatureSensorLM25::new(analog, 250);

  loop {
      let now = timer.millis();
      t.update(now, &mut adc);
      let (c_int, c_frac) = t.to_celsius(); // e.g. (23, 4) => 23.4°C
  }
  ```
</details>

---

## Tools (expand)

<details>
  <summary><b>Serial RAW audio streamer</b> — Tools/MP3_SERIAL_STREAM</summary>

Streams `*.raw` unsigned **u8 mono PCM** to a serial port at real-time speed.

**Run**
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

**Convert with ffmpeg**
  ```bash
  ffmpeg -i input.mp3 -ac 1 -ar 8000 -f u8 output.raw
  ```
</details>

---

## Troubleshooting

<details>
  <summary><b>I²C “no response” / device not detected</b></summary>

- Check **GND** (loose ground is #1).
- Verify SDA/SCL are on **A4/A5** (Uno/Nano).
- Try the other LCD address (`0x27` ↔ `0x3F`).
- Keep wires short; if unstable, reduce I²C speed (e.g. 100 kHz).
</details>

<details>
  <summary><b>LCD backlight on but no text</b></summary>

- Adjust **contrast** potentiometer (it can look blank otherwise).
- Confirm correct I²C address.
- Some PCF8574 backpacks use a different pin mapping (rare, but happens).
</details>

---

## Licenses

Dual-licensed under either of:

- Apache License 2.0 — `LICENSE-APACHE`
- MIT License — `LICENSE-MIT`

---

## Contributing

PRs/issues welcome. By default, contributions are dual-licensed under Apache-2.0 OR MIT.
