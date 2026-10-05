//! Sensirion SCD40 CO2/temperature/humidity driver over I2C.
//!
//! The driver starts periodic measurement mode once, then polls data-ready state
//! and reads a sample when available.

use embedded_hal::i2c::I2c;

const CMD_START_PERIODIC_MEASUREMENT: [u8; 2] = [0x21, 0xB1];
const CMD_READ_MEASUREMENT: [u8; 2] = [0xEC, 0x05];
const CMD_STOP_PERIODIC_MEASUREMENT: [u8; 2] = [0x3F, 0x86];
const CMD_GET_DATA_READY_STATUS: [u8; 2] = [0xE4, 0xB8];

/// First periodic sample appears after ~5 seconds.
const FIRST_MEASUREMENT_DELAY_MS: u32 = 5_000;
const STOP_TO_IDLE_DELAY_MS: u32 = 500;

/// SCD40 sensor state and cached readings.
pub struct Scd40Sensor {
    addr: u8,
    read_rate: u16,
    time: u32,
    started_at: u32,
    is_started: bool,
    restart_wait_started_at: Option<u32>,

    is_read: bool,
    is_command_error: bool,
    is_reading_error: bool,
    is_crc_error: bool,

    co2_ppm: u16,
    raw_t: u16,
    raw_hum: u16,
}

impl Scd40Sensor {
    /// Create a new SCD40 instance.
    ///
    /// `addr` is usually `0x62`.
    /// `read_rate` is polling interval (ms).
    pub fn new(addr: u8, read_rate: u16) -> Self {
        Self {
            addr,
            read_rate,
            time: 0,
            started_at: 0,
            is_started: false,
            restart_wait_started_at: None,
            is_read: false,
            is_command_error: false,
            is_reading_error: false,
            is_crc_error: false,
            co2_ppm: 0,
            raw_t: 0,
            raw_hum: 0,
        }
    }

    /// Periodic state-machine update.
    ///
    /// Starts periodic mode once and reads data periodically.
    pub fn update(&mut self, time: u32, i2c: &mut impl I2c) {
        self.is_read = false;
        self.is_command_error = false;
        self.is_reading_error = false;
        self.is_crc_error = false;

        if !self.ensure_started(time, i2c) {
            return;
        }

        if time.wrapping_sub(self.time) >= self.read_rate as u32 {
            self.read_measurement_if_ready(time, i2c);
        }
    }

    fn ensure_started(&mut self, time: u32, i2c: &mut impl I2c) -> bool {
        if let Some(wait_started_at) = self.restart_wait_started_at {
            if time.wrapping_sub(wait_started_at) < STOP_TO_IDLE_DELAY_MS {
                return false;
            }
            self.restart_wait_started_at = None;
        }

        if self.is_started {
            return true;
        }

        if self.start_periodic_measurement(time, i2c) {
            return true;
        }

        self.is_command_error = true;
        self.time = time;

        // Try to recover if the sensor stayed in periodic mode after MCU reset.
        if self.stop_periodic_measurement(i2c) {
            self.restart_wait_started_at = Some(time);
        }

        false
    }

    /// Start periodic measurement mode.
    fn start_periodic_measurement(&mut self, time: u32, i2c: &mut impl I2c) -> bool {
        if i2c.write(self.addr, &CMD_START_PERIODIC_MEASUREMENT).is_err() {
            return false;
        }

        self.is_started = true;
        self.started_at = time;
        self.time = time;
        self.restart_wait_started_at = None;
        true
    }

    /// Stop periodic measurement mode.
    pub fn stop_periodic_measurement(&mut self, i2c: &mut impl I2c) -> bool {
        if i2c.write(self.addr, &CMD_STOP_PERIODIC_MEASUREMENT).is_err() {
            return false;
        }

        self.is_started = false;
        self.restart_wait_started_at = None;
        true
    }

    fn read_measurement_if_ready(&mut self, time: u32, i2c: &mut impl I2c) {
        if time.wrapping_sub(self.started_at) < FIRST_MEASUREMENT_DELAY_MS {
            return;
        }

        if !self.is_data_ready(i2c) {
            self.time = time;
            return;
        }

        let mut b = [0u8; 9];
        if i2c.write_read(self.addr, &CMD_READ_MEASUREMENT, &mut b).is_err() {
            self.is_reading_error = true;
            self.time = time;
            return;
        }

        if !Self::word_crc_ok(&b[0..3]) || !Self::word_crc_ok(&b[3..6]) || !Self::word_crc_ok(&b[6..9]) {
            self.is_crc_error = true;
            self.time = time;
            return;
        }

        self.co2_ppm = u16::from_be_bytes([b[0], b[1]]);
        self.raw_t = u16::from_be_bytes([b[3], b[4]]);
        self.raw_hum = u16::from_be_bytes([b[6], b[7]]);
        self.time = time;
        self.is_read = true;
    }

    /// Check data-ready status.
    fn is_data_ready(&mut self, i2c: &mut impl I2c) -> bool {
        let mut b = [0u8; 3];
        if i2c.write_read(self.addr, &CMD_GET_DATA_READY_STATUS, &mut b).is_err() {
            self.is_reading_error = true;
            return false;
        }

        if !Self::word_crc_ok(&b) {
            self.is_crc_error = true;
            return false;
        }

        let status = u16::from_be_bytes([b[0], b[1]]);
        (status & 0x07FF) != 0
    }

    fn word_crc_ok(word_with_crc: &[u8]) -> bool {
        if word_with_crc.len() != 3 {
            return false;
        }

        Self::crc8(&word_with_crc[0..2]) == word_with_crc[2]
    }

    fn crc8(data: &[u8]) -> u8 {
        let mut crc = 0xFFu8;
        for &byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if (crc & 0x80) != 0 {
                    crc = (crc << 1) ^ 0x31;
                } else {
                    crc <<= 1;
                }
            }
        }

        crc
    }

    /// True only on update ticks where a new sample was read successfully.
    pub fn is_read(&self) -> bool { self.is_read }

    /// True if command write failed.
    pub fn is_command_error(&self) -> bool { self.is_command_error }

    /// True if read/status transaction failed.
    pub fn is_read_error(&self) -> bool { self.is_reading_error }

    /// True if CRC validation failed.
    pub fn is_crc_error(&self) -> bool { self.is_crc_error }

    /// Last CO2 concentration in ppm.
    pub fn get_co2_ppm(&self) -> u16 { self.co2_ppm }

    /// Temperature in Celsius as `(integer, fractional_2_digits)`.
    pub fn get_temp_celsius(&self) -> (i32, u32) {
        let t_x100 = ((self.raw_t as i32 * 17500 + 32768) / 65536) - 4500;

        let t_int = t_x100 / 100;
        let t_frac = (t_x100.abs() % 100) as u32;

        (t_int, t_frac)
    }

    /// Relative humidity as `(integer, fractional_2_digits)`.
    pub fn get_humidity(&self) -> (u32, u32) {
        let rh_x100 = (self.raw_hum as u32 * 10000 + 32768) / 65536;

        let rh_int = rh_x100 / 100;
        let rh_frac = rh_x100 % 100;

        (rh_int, rh_frac)
    }
}
