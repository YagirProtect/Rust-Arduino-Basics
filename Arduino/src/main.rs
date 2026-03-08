#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod modules;
mod std;

use arduino_hal::hal::usart::BaudrateArduinoExt;
use embedded_hal::i2c::I2c;
use panic_halt as _;
use modules::screen_lcd1602::screen_lcd1602::{EMode, ScreenLCD1602};
use crate::std::global_timer::GlobalTimer;
use crate::std::std::{enable_interrupts};
use core::fmt::Write;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);


    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    enable_interrupts();
    let timer = GlobalTimer::new(&dp.TC0);


    let mut screen = ScreenLCD1602::new(0x27, &mut i2c, EMode::Async);

    let mut last = timer.millis();
    let mut val = 0;
    loop {
        let now = timer.millis();

        val += 1;


        if (val == 0){
            screen.display_off(&mut i2c);
        }
        if (val == 10000) {
            write!(screen.get_line(), "nu ti i lox").unwrap();
            screen.display_on(&mut i2c);
            screen.print(&mut i2c);
        }
        if (val == 100000){
            screen.clear(&mut i2c);
        }
        if (val > 150000){
            val = 0;
        }

        screen.update(now, &mut i2c);

        last = now;
    }
}
