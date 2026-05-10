#![no_std]
#![no_main]

use arduino::modules::realtime_ds::realtime_ds3231::RealTimeDS3231;
use panic_halt as _;
use arduino::modules::realtime_ds::date_time::{build_datetime, DateTime};
use arduino::std::extensions::str_to_unumber::StrToNumberExt;

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

    let mut realtime = RealTimeDS3231::new(0x68, None);
    realtime.set_time(&mut i2c, build_datetime());

    loop {
        let alive = realtime.is_alive(&mut i2c);
        if (alive){
            let time = realtime.read_time(&mut i2c);
            if let Some(t) = time {
                ufmt::uwriteln!(&mut serial, "{}:{}:{} {}/{}/{}", t.hour, t.min, t.sec, t.day, t.month, t.year).unwrap();
            }

        }else{
            ufmt::uwriteln!(&mut serial, "module is dead").unwrap();
        }
    }
}
