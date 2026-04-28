//! SHT31 temperature/humidity sensor driver over I2C.
//!
//! The driver uses a two-phase read flow:
//! 1. Send measurement command (`0x2C, 0x06`)
//! 2. Wait a short conversion delay, then read 6 bytes
//!
//! This module keeps the latest raw values and exposes conversion helpers.

use arduino_hal::prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write};

/// Delay between command and data read for single-shot measurement.
const GET_DATA_DELAY_MS: u32 = 15;

/// SHT31 sensor state and cached readings.
pub struct TemperatureHumiditySensorSHT31 {
    addr: u8,
    is_read: bool,
    read_rate: u16,
    time: u32,
    is_auto_read: bool,

    collecting_time: u32,
    is_collecting_started: bool,
    is_collecting_error: bool,
    is_reading_error: bool,

    raw_t: u16,
    raw_hum: u16,
}

impl TemperatureHumiditySensorSHT31 {
    /// Create a new SHT31 instance.
    ///
    /// `addr` is typically `0x44` or `0x45`.
    /// `read_rate` is the auto-read interval in milliseconds.
    pub fn new(addr: u8, read_rate: u16, is_auto_read: bool) -> Self {
        Self {
            addr,
            read_rate,
            is_read: false,
            time: 0,
            is_auto_read,
            collecting_time: 0,
            is_collecting_started: false,
            is_collecting_error: false,
            is_reading_error: false,
            raw_t: 0,
            raw_hum: 0,
        }
    }

    /// Periodic state-machine update.
    ///
    /// In auto mode, starts a new measurement every `read_rate` ms.
    pub fn update(&mut self, time: u32, i2c: &mut arduino_hal::I2c) {
        if (self.is_collecting_started && !self.is_collecting_error && !self.is_reading_error) {
            self.read_sensor(time, i2c);
            return;
        } else {
            self.is_collecting_error = false;
            self.is_reading_error = false;
            self.is_read = false;
        }

        if (self.is_auto_read) {
            if (time.wrapping_sub(self.time) >= self.read_rate as u32) {
                self.read_sensor(time, i2c);
            }
        }
    }

    /// Manually run one measurement cycle (start or finish, depending on state).
    pub fn read_sensor(&mut self, time: u32, i2c: &mut arduino_hal::I2c) {
        if (self.is_collecting_started == false) {
            match i2c.write(self.addr, &[0x2C, 0x06]) {
                Ok(_) => {
                    self.is_collecting_started = true;
                    self.collecting_time = time;
                    self.time = time;
                }
                Err(_) => {
                    self.is_collecting_error = true;
                    self.is_collecting_started = false;
                    self.time = time;
                }
            }
        } else {
            let delta = time.wrapping_sub(self.collecting_time);

            if (delta >= GET_DATA_DELAY_MS) {
                let mut b = [0u8; 6];
                match i2c.read(self.addr, &mut b) {
                    Ok(_) => {
                        self.raw_t = u16::from_be_bytes([b[0], b[1]]);
                        self.raw_hum = u16::from_be_bytes([b[3], b[4]]);

                        self.is_read = true;
                    }
                    Err(_) => {
                        self.is_reading_error = true;
                        self.time = time;
                    }
                }

                self.is_collecting_started = false;
            }
        }
    }

    /// True only on update ticks where a new sample was read successfully.
    pub fn is_read(&self) -> bool { self.is_read }

    /// True if the data read phase failed.
    pub fn is_read_error(&self) -> bool { self.is_reading_error }

    /// True if sending the measurement command failed.
    pub fn is_collecting_error(&self) -> bool { self.is_collecting_error }

    /// True while conversion is in progress and waiting for read phase.
    pub fn is_collecting_process(&self) -> bool { self.is_collecting_started }

    /// Temperature in Celsius as `(integer, fractional_2_digits)`.
    pub fn get_temp_celsius(&self) -> (i32, u32) {
        let t_x100 = ((self.raw_t as i32 * 17500 + 32767) / 65535) - 4500;

        let t_int = t_x100 / 100;
        let t_frac = (t_x100.abs() % 100) as u32;

        (t_int, t_frac)
    }

    /// Temperature in Fahrenheit as `(integer, fractional_2_digits)`.
    pub fn get_temp_fahrenheit(&self) -> (i32, u32) {
        let c_x100 = ((self.raw_t as i32 * 17500 + 32767) / 65535) - 4500;
        let f_x100 = (c_x100 * 9) / 5 + 3200;

        let f_int = f_x100 / 100;
        let f_frac = (f_x100.abs() % 100) as u32;

        (f_int, f_frac)
    }

    /// Relative humidity as `(integer, fractional_2_digits)`.
    pub fn get_humidity(&self) -> (u32, u32) {
        let rh_x100 = (self.raw_hum as u32 * 10000 + 32767) / 65535;

        let rh_int = rh_x100 / 100;
        let rh_frac = rh_x100 % 100;

        (rh_int, rh_frac)
    }
}
