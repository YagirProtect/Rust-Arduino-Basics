//! BFS water sensor helper (analog output) with power gating.
//!
//! This driver is designed for common cheap "water level" / "rain" analog sensors.
//!
//! ## Why power is switched
//! Keeping these probes powered continuously can cause electrolysis/corrosion and noisy readings.
//! A common trick is to:
//! - set `power_pin` HIGH,
//! - read ADC,
//! - set `power_pin` LOW.
//!
//! The driver does exactly that in [`WaterSensorBFS::update`].
//!
//! ## Timing
//! Non-blocking scheduling is done using `read_rate` milliseconds and `wrapping_sub`.


use arduino_hal::adc::AdcChannel;
use arduino_hal::hal::Atmega;
use arduino_hal::pac::ADC as AdcPeriph;
use arduino_hal::port::mode::{Analog, Output};
use arduino_hal::port::{mode, Pin, PinOps};

const MAX_INPUT_VALUE: u16 = 1024;
/// BFS water sensor reader with power gating.
///
/// Powers the probe only during measurement:
/// `HIGH -> read ADC -> LOW`, which helps reduce corrosion and noise.
/// Use [`WaterSensorBFS::percent`] to get a normalized value.

pub struct WaterSensorBFS<PW, OT> {
    power_pin: Pin<mode::Output, PW>,
    output_pin: Pin<Analog, OT>,

    last_data: u16,
    read_rate: u32,
    time: u32,


    is_read: bool
}

impl<PW, OT> WaterSensorBFS<PW, OT>
where
    PW: PinOps,
    OT: PinOps,
    Pin<Analog, OT>: AdcChannel<Atmega, AdcPeriph>,
{
    /// Create a new water sensor reader (power pin + analog output + read rate in ms).
    pub fn new(power_pin: Pin<Output, PW>, output_pin: Pin<Analog, OT>, read_rate: u32) -> Self {
        Self{
            power_pin,
            output_pin,
            last_data: 0,
            read_rate,
            time: 0,
            is_read: false,
        }
    }

    /// Perform a powered measurement if `read_rate` elapsed.
    ///
    /// Sets power HIGH, reads ADC, then sets power LOW.

    pub fn update(&mut self, time: u32, adc: &mut arduino_hal::Adc) {
        if (time.wrapping_sub(self.time) >= self.read_rate as u32){
            self.power_pin.set_high();
            self.last_data = adc.read_blocking(&self.output_pin);
            self.power_pin.set_low();

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

    /// Normalized value in `[0,1]` using `MAX_INPUT_VALUE` as full scale.
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
//     let mut power = pins.d7.into_output();
//     let analog0 = pins.a0.into_analog_input(&mut adc);
//
//     let mut water_sensor = WaterSensorBFS::new(power, analog0, 500);
//
//
//     loop {
//         let now = timer.millis();
//         water_sensor.update(now, &mut adc);
//
//         if (water_sensor.is_read()){
//             writeln!(io.str(), "Water Level: {}", water_sensor.last_data());
//             io.log();
//         }
//     }
// }
