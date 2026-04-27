use arduino_hal::adc::AdcChannel;
use arduino_hal::hal::Atmega;
use arduino_hal::pac::ADC as AdcPeriph;
use arduino_hal::port::mode::Analog;
use arduino_hal::port::{Pin, PinOps};

const MAX_INPUT_VALUE: u16 = 1023;
const RAW_SAMPLE_COUNT: usize = 7;
const DEFAULT_BASELINE_SAMPLES: u16 = 30;

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
    baseline_sum: u32,
    baseline_samples_done: u16,
    baseline_samples_target: u16,
    baseline_ready: bool,
}

impl<AO> Mq135Sensor<AO>
where
    AO: PinOps,
{
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
            baseline_sum: 0,
            baseline_samples_done: 0,
            baseline_samples_target: DEFAULT_BASELINE_SAMPLES,
            baseline_ready: false,
        }
    }

    pub fn is_read(&self) -> bool {
        self.is_read
    }

    pub fn set_auto_read(&mut self, state: bool) {
        self.is_auto_read = state;
    }

    pub fn set_read_rate(&mut self, read_rate: u16) {
        self.read_rate = read_rate;
    }

    pub fn last_raw(&self) -> u16 {
        self.last_raw
    }

    pub fn smoothed_raw(&self) -> u16 {
        self.smoothed_raw
    }

    pub fn baseline_raw(&self) -> u16 {
        self.baseline_raw
    }

    pub fn baseline_ready(&self) -> bool {
        self.baseline_ready
    }

    pub fn baseline_progress(&self) -> (u16, u16) {
        (self.baseline_samples_done, self.baseline_samples_target)
    }

    pub fn reset_baseline_learning(&mut self) {
        self.baseline_raw = 0;
        self.baseline_sum = 0;
        self.baseline_samples_done = 0;
        self.baseline_ready = false;
    }

    pub fn set_baseline_sample_count(&mut self, count: u16) {
        self.baseline_samples_target = if count == 0 { 1 } else { count };
        self.reset_baseline_learning();
    }

    pub fn get_adc_percent(&self) -> (u32, u32) {
        let percent_x100 = self.smoothed_raw as u32 * 10000 / MAX_INPUT_VALUE as u32;
        let int = percent_x100 / 100;
        let frac = percent_x100 % 100;
        (int, frac)
    }

    pub fn delta_from_baseline(&self) -> i32 {
        if !self.baseline_ready {
            return 0;
        }

        self.smoothed_raw as i32 - self.baseline_raw as i32
    }

    pub fn positive_delta_from_baseline(&self) -> u16 {
        let delta = self.delta_from_baseline();
        if delta <= 0 { 0 } else { delta as u16 }
    }

    pub fn is_above_baseline(&self, threshold: u16) -> bool {
        self.baseline_ready && self.positive_delta_from_baseline() >= threshold
    }
}

impl<AO> Mq135Sensor<AO>
where
    AO: PinOps,
    Pin<Analog, AO>: AdcChannel<Atmega, AdcPeriph>,
{
    pub fn update(&mut self, time: u32, adc: &mut arduino_hal::Adc) {
        self.is_read = false;

        if self.is_auto_read && time.wrapping_sub(self.time) >= self.read_rate as u32 {
            self.read_sensor(time, adc);
        }
    }

    pub fn read_sensor(&mut self, time: u32, adc: &mut arduino_hal::Adc) {
        let raw = self.read_stable_raw(adc);

        self.last_raw = raw;

        if self.is_first_read {
            self.smoothed_raw = raw;
            self.is_first_read = false;
        } else {
            // EMA: 75% старое + 25% новое
            self.smoothed_raw =
                ((self.smoothed_raw as u32 * 3 + raw as u32) / 4) as u16;
        }

        if !self.baseline_ready {
            self.baseline_sum += self.smoothed_raw as u32;
            self.baseline_samples_done += 1;
            self.baseline_raw = (self.baseline_sum / self.baseline_samples_done as u32) as u16;

            if self.baseline_samples_done >= self.baseline_samples_target {
                self.baseline_ready = true;
            }
        }

        self.time = time;
        self.is_read = true;
    }

    fn read_stable_raw(&mut self, adc: &mut arduino_hal::Adc) -> u16 {
        let mut samples = [0u16; RAW_SAMPLE_COUNT];

        let mut i = 0;
        while i < RAW_SAMPLE_COUNT {
            samples[i] = adc.read_blocking(&mut self.output_pin);
            i += 1;
        }

        Self::median(samples)
    }

    fn median(mut values: [u16; RAW_SAMPLE_COUNT]) -> u16 {
        let mut i = 1;
        while i < RAW_SAMPLE_COUNT {
            let key = values[i];
            let mut j = i;

            while j > 0 && values[j - 1] > key {
                values[j] = values[j - 1];
                j -= 1;
            }

            values[j] = key;
            i += 1;
        }

        values[RAW_SAMPLE_COUNT / 2]
    }
}