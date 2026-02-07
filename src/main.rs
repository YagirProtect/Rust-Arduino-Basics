#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
use core::fmt::Write;
use panic_halt as _;
use crate::joystick::Joystick;

mod std;
mod joystick;

use crate::std::global_timer::GlobalTimer;
use crate::std::io::IoUno;
use crate::std::std::enable_interrupts;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let pins: arduino_hal::Pins = arduino_hal::pins!(dp);
    let timer = GlobalTimer::new(&dp.TC0);
    let mut io = IoUno::new(dp.USART0, pins.d0, pins.d1, 115200);
    enable_interrupts();


    let analog0 = pins.a0.into_analog_input(&mut adc);
    let analog1 = pins.a1.into_analog_input(&mut adc);
    let button = pins.d7.into_pull_up_input();
    let mut joystick = Joystick::new(Some(analog0), Some(analog1), Some(button), 8);

    loop {
        let now = timer.millis();
        joystick.update(now, &mut adc);

        write!(io.str(), "{}, {}, {}", joystick.x_raw(), joystick.y_raw(), joystick.button()).unwrap();
        io.log();
        
        if (joystick.button_pressed()){
            write!(io.str(), "pressed!!!").unwrap();
            io.log();
        }
    }
}
