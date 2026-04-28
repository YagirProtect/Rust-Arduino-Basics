#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
use panic_halt as _;
mod modules;
mod std;

use crate::modules::realtime_ds1302::{DateTime};
use crate::modules::temp_hum_sht31::TemperatureHumiditySensorSHT31;
use crate::std::global_timer::GlobalTimer;
use crate::std::std::enable_interrupts;
use arduino_hal::hal::usart::BaudrateArduinoExt;
use core::fmt::Write;
use embedded_hal::i2c::I2c;
use modules::screen_lcd1602::screen_lcd1602::{EMode, RecoverModule, ScreenLCD1602};
use crate::modules::button::Button;
use crate::std::extensions::str_to_unumber::StrToNumberExt;

use arduino_hal::adc::AdcChannel;
use arduino_hal::port::{PinOps};
use crate::modules::heartbeat_diode::HeartbeatDiode;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50_000,
    );


    let timer = GlobalTimer::new(&dp.TC0);

    let mode_button = Button::new(pins.d4.into_pull_up_input());

    let mut screen = ScreenLCD1602::new(0x27, &mut i2c, EMode::Linear, Some(RecoverModule::recommended()));
    let mut temp_hum = TemperatureHumiditySensorSHT31::new(0x44, 1000, true);
    let mut heartbeat_diode = HeartbeatDiode::new(pins.d5.into_output(), 1000);

    screen.display_on(&mut i2c);

    enable_interrupts();

    let mut display_work_time = 0;

    let mut override_display_state = true;

    let mut last_press_btn_time = 0;

    let mut last_button_state = false;

    loop {
        let now = timer.millis();
        screen.set_now(now);


        read_button_and_change_state(&mut i2c, &mut screen, &mode_button, &mut display_work_time, &mut override_display_state, &mut last_press_btn_time, &mut last_button_state, now);
        read_temperature(&mut i2c, &mut screen, &mut temp_hum, now);
        try_draw_on_screen(&mut i2c, &mut screen, &mut temp_hum, &mut display_work_time);
        update_screen_by_timer(&mut i2c, &mut screen, override_display_state);

        
        
        heartbeat_diode.update(now);
        if screen.need_recovery() {
            screen.recover(&mut i2c);
        }
        screen.update(now, &mut i2c);
    }
}

fn read_button_and_change_state(mut i2c:  &mut arduino_hal::I2c, screen: &mut ScreenLCD1602, mode_button: &Button<impl PinOps>, display_work_time: &mut i32, override_display_state: &mut bool, last_press_btn_time: &mut u32, last_button_state: &mut bool, now: u32) {
    if (mode_button.is_pressed()) {

        if (*last_button_state != mode_button.is_pressed()) {
            *display_work_time = 0;
            screen.display_on(&mut i2c);

            if (now.wrapping_sub(*last_press_btn_time) <= 500) {
                *override_display_state = !*override_display_state;
            }

            *last_press_btn_time = now;
            *last_button_state = true;
        }
    } else {
        *last_button_state = false;
    }
}

fn try_draw_on_screen(
    i2c: &mut arduino_hal::I2c,
    screen: &mut ScreenLCD1602,
    temp_hum: &mut TemperatureHumiditySensorSHT31,
    display_work_time: &mut i32,
)
{
    if temp_hum.is_read() {
        let temp = temp_hum.get_temp_celsius();
        let hum = temp_hum.get_humidity();

        let is_writed = write!(
            screen.get_line(),
            "Temp: {:02}.{:02} C\nHum : {:02}.{:02} %",
            temp.0, temp.1,
            hum.0, hum.1,
        ).is_ok();

        if is_writed {
            screen.print(i2c);
        }

        *display_work_time += 1;
    }
}

fn read_temperature(
    mut i2c: &mut arduino_hal::I2c,
    screen: &mut ScreenLCD1602,
    temp_hum: &mut TemperatureHumiditySensorSHT31,
    now: u32)
{
    if (screen.is_display_on()) {
        temp_hum.update(now, &mut i2c);
    }
}

fn update_screen_by_timer(
    mut i2c: &mut arduino_hal::I2c,
    screen: &mut ScreenLCD1602,
    override_display_state: bool
) {
    if (override_display_state == false) {
        if (screen.is_display_on()){
            screen.clear(&mut i2c);
            screen.display_off(&mut i2c, false);
        }
    }else{
        if (!screen.is_display_on()){
            screen.display_on(&mut i2c);
        }
    }
}
