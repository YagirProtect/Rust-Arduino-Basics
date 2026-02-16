#![no_std]
#![no_main]

use panic_halt as _;
use ufmt::uwriteln;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Serial monitor: 115200
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    uwriteln!(&mut serial, "SHT31 test start").ok();

    // I2C на Uno/Nano: SDA=A4, SCL=A5
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,                      // 100 kHz
    );

    // Обычно SHT31: 0x44 (иногда 0x45)
    let addr: u8 = 0x44;

    loop {
        match read_sht31_blocking(&mut i2c, addr) {
            Ok((t_x100, rh_x100)) => {
                // T: i16, RH: u16. Печатаем без {:02} — вручную двумя цифрами.
                let t_int = t_x100 / 100;
                let t_frac = (t_x100.abs() as u16) % 100;
                let t_f1 = t_frac / 10;
                let t_f2 = t_frac % 10;

                let rh_int = rh_x100 / 100;
                let rh_frac = rh_x100 % 100;
                let rh_f1 = rh_frac / 10;
                let rh_f2 = rh_frac % 10;

                uwriteln!(
                    &mut serial,
                    "T={}.{}{} C | RH={}.{}{} %",
                    t_int, t_f1, t_f2,
                    rh_int, rh_f1, rh_f2
                ).ok();
            }
            Err(_) => {
                uwriteln!(&mut serial, "Read error. Try addr 0x45 or check wiring.").ok();
            }
        }

        arduino_hal::delay_ms(1000);
    }
}

/// Returns (temp_c_x100, rh_x100)
fn read_sht31_blocking<I2C>(i2c: &mut I2C, addr: u8) -> Result<(i16, u16), ()>
where
    I2C: embedded_hal::i2c::I2c<embedded_hal::i2c::SevenBitAddress>,
{
    // Single shot, high repeatability: 0x2C 0x06
    i2c.write(addr, &[0x2C, 0x06]).map_err(|_| ())?;
    arduino_hal::delay_ms(15);

    let mut b = [0u8; 6];
    i2c.read(addr, &mut b).map_err(|_| ())?;

    let raw_t = u16::from_be_bytes([b[0], b[1]]);
    let raw_rh = u16::from_be_bytes([b[3], b[4]]);

    // T = -45 + 175 * raw / 65535
    // RH = 100 * raw / 65535
    let t_x100: i16 = (((raw_t as i32 * 17500 + 32767) / 65535) - 4500) as i16;
    let rh_x100: u16 = ((raw_rh as u32 * 10000 + 32767) / 65535) as u16;

    Ok((t_x100, rh_x100))
}
