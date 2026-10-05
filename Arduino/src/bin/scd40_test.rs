#![no_std]
#![no_main]

use arduino::modules::scd40_sensor::Scd40Sensor;
use arduino::std::global_timer::GlobalTimer;
use arduino::std::std::enable_interrupts;
use panic_halt as _;
use ufmt::uwriteln;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50_000,
    );

    let timer = GlobalTimer::new(&dp.TC0);
    enable_interrupts();

    let mut scd40 = Scd40Sensor::new(0x62, 1000);
    let mut last_log_time = 0u32;

    uwriteln!(&mut serial, "SCD40 test started").unwrap();

    loop {
        let now = timer.millis();
        scd40.update(now, &mut i2c);

        if scd40.is_read() {
            let co2 = scd40.get_co2_ppm();
            let temp = scd40.get_temp_celsius();
            let hum = scd40.get_humidity();

            uwriteln!(
                &mut serial,
                "CO2={}ppm T={}.{}C RH={}.{}%",
                co2,
                temp.0,
                temp.1,
                hum.0,
                hum.1
            ).unwrap();

            last_log_time = now;
        } else if now.wrapping_sub(last_log_time) >= 1000 {
            if scd40.is_command_error() {
                uwriteln!(&mut serial, "SCD40 command error").unwrap();
                last_log_time = now;
            } else if scd40.is_read_error() {
                uwriteln!(&mut serial, "SCD40 read error").unwrap();
                last_log_time = now;
            } else if scd40.is_crc_error() {
                uwriteln!(&mut serial, "SCD40 crc error").unwrap();
                last_log_time = now;
            }
        }
    }
}
