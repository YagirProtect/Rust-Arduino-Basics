//! Light sensor (photoresistor / LDR) with optional power gating.
//!
//! Reads an analog light level from an LDR + resistor divider.
//!
//! ## Power gating (optional)
//! Some sensors can be powered from a digital pin to reduce idle power or to avoid
//! corrosion/electrolysis on exposed probes. If `power_pin` is `None`, the sensor is assumed
//! to be always powered.
//!
//! ## Timing
//! Call [`LightSensorResistor::update`] with a monotonic millisecond timestamp.
//! When `read_rate` elapsed, the sensor performs one blocking ADC read and sets `is_read=true`.
//!
//! ## Calibration
//! `percent()` returns `last_data / MAX_INPUT_VALUE`. The constant is project-specific;
//! adjust it to match your divider/Vref/expected max reading.


use arduino_hal::adc::AdcChannel;
use arduino_hal::hal::Atmega;
use arduino_hal::pac::ADC as AdcPeriph;
use arduino_hal::port::mode::{Analog, Output};
use arduino_hal::port::{mode, Pin, PinOps};

const MAX_INPUT_VALUE: u16 = 512;
/// Photoresistor (LDR) analog reader with optional power control.
///
/// If `power_pin` is provided, you can turn the sensor on/off (useful for power saving).
/// Call [`LightSensorResistor::update`] periodically; when a read occurs `is_read()` is true.

pub struct LightSensorResistor<PW, OT> {
    power_pin: Option<Pin<mode::Output, PW>>,
    output_pin: Pin<Analog, OT>,

    last_data: u16,
    read_rate: u32,
    time: u32,


    is_read: bool
}

impl<PW, OT> LightSensorResistor<PW, OT>
where
    PW: PinOps,
    OT: PinOps,
    Pin<Analog, OT>: AdcChannel<Atmega, AdcPeriph>,
{
    /// Create a new light sensor reader.
    ///
    /// - `power_pin`: optional output pin to power the divider (set HIGH to enable)
    /// - `output_pin`: analog input pin connected to the divider output
    /// - `read_rate`: minimum interval between reads, in milliseconds

    pub fn new(power_pin: Option<Pin<Output, PW>>, output_pin: Pin<Analog, OT>, read_rate: u32) -> Self {
        Self{
            power_pin,
            output_pin,
            last_data: 0,
            read_rate,
            time: 0,
            is_read: false,
        }
    }

    /// Enable/disable sensor power (if a power pin was provided).
    pub fn set_power(&mut self, state: bool){
        if let Some(pin) = self.power_pin.as_mut() {
            if (state) {
                pin.set_high();
            }else {
                pin.set_low();
            }
        }
    }

    /// Sample the ADC if `read_rate` elapsed.
    ///
    /// Sets `is_read=true` on the tick when a reading is performed.

    pub fn update(&mut self, time: u32, adc: &mut arduino_hal::Adc) {
        if (time.wrapping_sub(self.time) >= self.read_rate as u32){
            self.last_data = adc.read_blocking(&self.output_pin);
            self.time = time;
            self.is_read = true;
        }else{
            self.is_read = false;
        }
    }

    /// True only on the tick when a new reading was taken.
    pub fn is_read(&self) -> bool{
        self.is_read
    }

    pub fn last_data(&self) -> u16 {
        self.last_data
    }

    /// Normalized value in `[0,1]` using `MAX_INPUT_VALUE` as "full scale".
    ///
    /// Adjust `MAX_INPUT_VALUE` to match your divider/Vref.

    pub fn percent(&self)-> f32{
        return self.last_data as f32 / MAX_INPUT_VALUE as f32;
    }
}



//Example how to use
// #[arduino_hal::entry]
// fn main() -> ! {
//     let dp = arduino_hal::Peripherals::take().unwrap();
//     let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
//
//     let pins: arduino_hal::Pins = arduino_hal::pins!(dp);
//     let timer = GlobalTimer::new(&dp.TC0);
//     let mut io = IoUno::new(dp.USART0, pins.d0, pins.d1, 115200);
//     enable_interrupts();
//
//
//     let analog0 = pins.a0.into_analog_input(&mut adc);
//     let mut power = pins.d7.into_output();
//
//     let mut light_sensor = LightSensor::new(Some(power), analog0, 500);
//
//     light_sensor.set_power(true);
//     loop {
//         let now = timer.millis();
//
//         light_sensor.update(now, &mut adc);
//
//         if (light_sensor.is_read()) {
//             if (light_sensor.percent() > 0.3){
//                 light_sensor.set_power(false);
//             }
//         }
//         writeln!(io.str(), "Light Level: {}", light_sensor.last_data());
//         io.log();
//     }
// }
