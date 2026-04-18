#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
use panic_halt as _;
mod modules;
mod std;

use crate::modules::realtime_ds1302::{DateTime, RealTimeDS1302};
use crate::modules::temp_hum_sht31::TemperatureHumiditySensorSHT31;
use crate::std::global_timer::GlobalTimer;
use crate::std::std::enable_interrupts;
use arduino_hal::hal::usart::BaudrateArduinoExt;
use core::fmt::Write;
use embedded_hal::i2c::I2c;
use ufmt::uwriteln;
use modules::screen_lcd1602::screen_lcd1602::{EMode, ScreenLCD1602};
use crate::std::extensions::str_to_unumber::StrToNumberExt;

fn build_datetime() -> DateTime {
    DateTime {
        sec: env!("BUILD_SEC").to_u8(),
        min: env!("BUILD_MIN").to_u8(),
        hour: env!("BUILD_HOUR").to_u8(),
        day: env!("BUILD_DAY").to_u8(),
        day_in_week: env!("BUILD_WEEKDAY").to_u8(),
        month: env!("BUILD_MONTH").to_u8(),
        year: (env!("BUILD_YEAR").to_u16() - 2000) as u8,
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );


    enable_interrupts();
    let timer = GlobalTimer::new(&dp.TC0);


    let mut screen = ScreenLCD1602::new(0x27, &mut i2c, EMode::Async);
    let mut temp_hum = TemperatureHumiditySensorSHT31::new(0x44, 1000, true);


    let clk = pins.d5.into_output();
    let dat = pins.d6.into_output();
    let rst = pins.d7.into_output();
    let mut real_time = RealTimeDS1302::new(clk, dat, rst);


    // real_time.set_time(build_datetime());
    
    let mut last = timer.millis();
    loop {
        let now = timer.millis();

        temp_hum.update(now, &mut i2c);

        // let temp = temp_hum.get_temp_celsius();
        // let hum = temp_hum.get_humidity();
        // uwriteln!(
        //             &mut serial,
        //             "Debug: T={}.{} C | RH={}.{} % | is_read={}, | is_process = {} | is_read_error = {} | is_collect_error = {}",
        //             temp.0, temp.1,
        //             hum.0, hum.1, temp_hum.is_read(), temp_hum.is_collecting_process(), temp_hum.is_read_error(), temp_hum.is_collecting_error()
        //         ).ok();



        if (temp_hum.is_read()) {
            let temp = temp_hum.get_temp_celsius();
            let hum = temp_hum.get_humidity();


            write!(screen.get_line(), "Temp: {}.{} C\nHum : {}.{} %", temp.0, temp.1, hum.0, hum.1).unwrap();
            screen.print(&mut i2c);


            let time = real_time.read_time();
            uwriteln!(
                &mut serial,
                "Debug: Time = {}:{}:{}, {}-{}-{}",
                time.hour, time.min, time.sec, time.day, time.month, time.year
            ).ok();

        }

        screen.update(now, &mut i2c);

        last = now;
    }
}
