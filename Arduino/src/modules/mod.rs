//! Project modules.
//!
//! This folder contains small **no_std-friendly** drivers and helpers for Arduino-class
//! microcontrollers (ATmega328P / Uno / Nano).
//!
//! The modules are intentionally minimal:
//! - **Sensors** expose `new(...)`, `update(now, adc)` and getters for last readings.
//! - **Actuators / displays** provide simple high-level helpers plus low-level access when needed.
//!
//! Most modules follow a non-blocking pattern using `millis()` timestamps and `wrapping_sub`
//! to stay safe across timer overflows.


pub mod water_sensor_bfs;
pub mod light_sensor_resistor;
pub mod temperature_sensor_lm25;
pub mod joystick_hw504;
pub mod screen_lcd1602;
pub mod temp_hum_sht31;
pub mod realtime_ds1302;
pub mod button;