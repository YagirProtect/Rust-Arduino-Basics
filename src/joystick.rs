use arduino_hal::port::{mode, Pin, PinOps};
use arduino_hal::adc::{AdcChannel, Channel};
use arduino_hal::hal::Atmega;
use arduino_hal::pac::ADC as AdcPeriph;
use arduino_hal::port::mode::{Analog, PullUp};

const MAX_INPUT_VALUE: u16 = 1024;

pub struct Joystick<X, Y, SW>{
    x: u16,
    y: u16,
    tap: bool,

    read_rate: u16,

    x_pin:   Option<Pin<Analog, X>>,
    y_pin:   Option<Pin<Analog, Y>>,
    tap_pin: Option<Pin<mode::Input<mode::PullUp>, SW>>,

    tap_pressed: bool,
    time: u32,
}

impl<X, Y, SW> Joystick<X, Y, SW>
where
    Pin<Analog, X>: AdcChannel<Atmega, AdcPeriph>,
    Pin<Analog, Y>: AdcChannel<Atmega, AdcPeriph>,
    SW: PinOps,
{

    pub fn new (pin_x: Option<Pin<Analog, X>>, pin_y: Option<Pin<Analog, Y>>, pin_tap: Option<Pin<mode::Input<PullUp>, SW>>, read_rate: u16) -> Self{

        Self {
            x: MAX_INPUT_VALUE / 2,
            y: MAX_INPUT_VALUE / 2,
            tap: false,

            x_pin: pin_x,
            y_pin: pin_y,
            tap_pin: pin_tap,
            read_rate: read_rate,
            time: 0,
            tap_pressed: false,
        }
    }


    pub fn update(&mut self, time: u32, adc: &mut arduino_hal::Adc){
        self.tap_pressed = false;
        if (time.wrapping_sub(self.time) >= self.read_rate as u32){

            if let Some(pin) = self.tap_pin.as_mut() {
                let prev = self.tap;
                self.tap = pin.is_low();
                if (prev == false && self.tap){
                    self.tap_pressed = self.tap;
                }
            }
            if let Some(pin) = self.x_pin.as_mut() {
                self.x = adc.read_blocking(pin);
            }
            if let Some(pin) = self.y_pin.as_mut() {
                self.y = adc.read_blocking(pin);
            }

            self.time = time;
        }
    }

    pub fn x_raw(&self) -> u16 {
        self.x
    }
    pub fn y_raw(&self) -> u16 {
        self.y
    }
    pub fn button(&self) -> bool {self.tap}
    pub fn button_pressed(&self) -> bool {self.tap_pressed}
}