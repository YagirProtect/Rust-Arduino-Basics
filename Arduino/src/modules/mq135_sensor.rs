//! MQ-135 analog sensor helper.
//!
//! This module is intentionally lightweight:
//! - one ADC read per sample (no sample buffer/median sort),
//! - integer-only smoothing and baseline learning (no `f32`),
//! - periodic auto-read based on `millis()`.
//!
//! Notes about "percent":
//! `get_adc_percent()` returns ADC level percent (0..100%), not ppm.
//! Real gas concentration requires calibration curve and temperature/humidity compensation.

use arduino_hal::adc::AdcChannel;
use arduino_hal::hal::Atmega;
use arduino_hal::pac::ADC as AdcPeriph;
use arduino_hal::port::mode::Analog;
use arduino_hal::port::{Pin, PinOps};

/// 10-bit AVR ADC full scale.
const MAX_INPUT_VALUE: u16 = 1023;
/// Default count of startup samples used to learn baseline.
const DEFAULT_BASELINE_SAMPLES: u16 = 30;

/// MQ-135 runtime state and processed values.
pub struct Mq135Sensor<AO> {
    output_pin: Pin<Analog, AO>,

    is_read: bool,
    read_rate: u16,
    time: u32,
    is_auto_read: bool,

    last_raw: u16,
    smoothed_raw: u16,
    is_first_read: bool,

    baseline_raw: u16,
    baseline_samples_done: u16,
    baseline_samples_target: u16,
    baseline_ready: bool,
}

impl<AO> Mq135Sensor<AO>
where
    AO: PinOps,
{
    /// Create sensor helper.
    ///
    /// `read_rate` is in milliseconds.
    pub fn new(output_pin: Pin<Analog, AO>, read_rate: u16, is_auto_read: bool) -> Self {
        Self {
            output_pin,
            is_read: false,
            read_rate,
            time: 0,
            is_auto_read,
            last_raw: 0,
            smoothed_raw: 0,
            is_first_read: true,
            baseline_raw: 0,
            baseline_samples_done: 0,
            baseline_samples_target: DEFAULT_BASELINE_SAMPLES,
            baseline_ready: false,
        }
    }

    /// True only on ticks where a new sample was read.
    pub fn is_read(&self) -> bool {
        self.is_read
    }

    /// Enable/disable periodic auto-read mode.
    pub fn set_auto_read(&mut self, state: bool) {
        self.is_auto_read = state;
    }

    /// Set auto-read period (milliseconds).
    pub fn set_read_rate(&mut self, read_rate: u16) {
        self.read_rate = read_rate;
    }

    /// Last raw ADC value (0..1023).
    pub fn last_raw(&self) -> u16 {
        self.last_raw
    }

    /// Smoothed ADC value after EMA filter.
    pub fn smoothed_raw(&self) -> u16 {
        self.smoothed_raw
    }

    /// Learned baseline ADC value.
    pub fn baseline_raw(&self) -> u16 {
        self.baseline_raw
    }

    /// Baseline is considered valid after `baseline_samples_target` samples.
    pub fn baseline_ready(&self) -> bool {
        self.baseline_ready
    }

    /// Returns `(done, target)` for baseline learning progress.
    pub fn baseline_progress(&self) -> (u16, u16) {
        (self.baseline_samples_done, self.baseline_samples_target)
    }

    /// Reset baseline learning process.
    pub fn reset_baseline_learning(&mut self) {
        self.baseline_raw = 0;
        self.baseline_samples_done = 0;
        self.baseline_ready = false;
    }

    /// Set number of samples used for baseline and restart learning.
    pub fn set_baseline_sample_count(&mut self, count: u16) {
        self.baseline_samples_target = if count == 0 { 1 } else { count };
        self.reset_baseline_learning();
    }

    /// ADC level in percent as `(int, frac_2_digits)`.
    pub fn get_adc_percent(&self) -> (u32, u32) {
        let percent_x100 = self.smoothed_raw as u32 * 10000 / MAX_INPUT_VALUE as u32;
        let int = percent_x100 / 100;
        let frac = percent_x100 % 100;
        (int, frac)
    }

    /// Signed delta from baseline.
    pub fn delta_from_baseline(&self) -> i32 {
        if !self.baseline_ready {
            return 0;
        }

        self.smoothed_raw as i32 - self.baseline_raw as i32
    }

    /// Positive part of `delta_from_baseline()`.
    pub fn positive_delta_from_baseline(&self) -> u16 {
        let delta = self.delta_from_baseline();
        if delta <= 0 { 0 } else { delta as u16 }
    }

    /// Convenience threshold check against positive baseline delta.
    pub fn is_above_baseline(&self, threshold: u16) -> bool {
        self.baseline_ready && self.positive_delta_from_baseline() >= threshold
    }
}

impl<AO> Mq135Sensor<AO>
where
    AO: PinOps,
    Pin<Analog, AO>: AdcChannel<Atmega, AdcPeriph>,
{
    /// Periodic update. In auto mode reads once per `read_rate` ms.
    pub fn update(&mut self, time: u32, adc: &mut arduino_hal::Adc) {
        self.is_read = false;

        if self.is_auto_read && time.wrapping_sub(self.time) >= self.read_rate as u32 {
            self.read_sensor(time, adc);
        }
    }

    /// Perform one sensor read and update derived values.
    pub fn read_sensor(&mut self, time: u32, adc: &mut arduino_hal::Adc) {
        let raw = adc.read_blocking(&mut self.output_pin);
        self.last_raw = raw;

        if self.is_first_read {
            self.smoothed_raw = raw;
            self.is_first_read = false;
        } else {
            // EMA with alpha = 1/4:
            // y_n = (3/4)*y_{n-1} + (1/4)*x_n
            //
            // Why "*3 + raw) / 4":
            // - "3" and "4" are integer form of 0.75 and 0.25.
            // - power-of-two divisor (4) keeps math fast on AVR (shift-friendly).
            // - keeps 75% inertia from previous value and 25% from new sample.
            self.smoothed_raw = ((self.smoothed_raw as u32 * 3 + raw as u32) / 4) as u16;
        }

        self.update_baseline();
        self.time = time;
        self.is_read = true;
    }

    fn update_baseline(&mut self) {
        if self.baseline_ready {
            return;
        }

        self.baseline_samples_done = self.baseline_samples_done.saturating_add(1);

        if self.baseline_samples_done == 1 {
            self.baseline_raw = self.smoothed_raw;
        } else {
            // Running average update:
            // baseline_n = baseline_{n-1} + (value_n - baseline_{n-1}) / n
            //
            // Why this formula:
            // - equivalent to arithmetic mean over n samples,
            // - no wide cumulative sum required,
            // - stable integer-only update on MCU.
            let n = self.baseline_samples_done as i32;
            let baseline = self.baseline_raw as i32;
            let value = self.smoothed_raw as i32;
            let next = baseline + (value - baseline) / n;
            self.baseline_raw = next as u16;
        }

        if self.baseline_samples_done >= self.baseline_samples_target {
            self.baseline_ready = true;
        }
    }
}
