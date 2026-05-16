#![no_std]
#![no_main]

use arduino::modules::realtime_ds::realtime_ds3231::RealTimeDS3231;
use panic_halt as _;
use arduino::modules::realtime_ds::date_time::{DateTime};
use arduino::modules::realtime_ds::realtime_ds3231_alarm::{Alarm1DateTime, AlarmModule};

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
    
    let mut led = pins.d2.into_output();

    let mut realtime = RealTimeDS3231::new(0x68, Some(AlarmModule::recommended()));
    realtime.set_time(&mut i2c, DateTime::now());

    if let Some(a) = realtime.alarms_mut() {
        a.set_alarm1_exact(&mut i2c, DateTime::now().add_seconds(30));
        a.enable_interrupt_pin(&mut i2c, true);    // INTCN
        a.enable_alarm1_interrupt(&mut i2c, true); // A1IE
    }

    loop {
        let alive = realtime.is_alive(&mut i2c);
        if (alive){
            let time = realtime.read_time(&mut i2c);
            if let Some(t) = time {
                ufmt::uwriteln!(&mut serial, "{}:{}:{} {}/{}/{}", t.hour, t.min, t.sec, t.day, t.month, t.year).unwrap();
            }

            if let Some(a) = realtime.alarms_mut() {
                if a.is_alarm1_triggered(&mut i2c) == Some(true) {
                    led.set_high();          // зажечь
                    a.clear_flags(&mut i2c); // обязательно
                    ufmt::uwriteln!(&mut serial, "ALARM").unwrap();
                }
            }
            
            
        }else{
            ufmt::uwriteln!(&mut serial, "module is dead").unwrap();
        }
    }
}
