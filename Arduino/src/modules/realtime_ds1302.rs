//! DS1302 real-time clock bit-bang driver.
//!
//! Interface lines:
//! - `CLK`: serial clock
//! - `DAT`: bidirectional data line
//! - `RST`: chip enable
//!
//! Registers are read/written in BCD format and converted to decimal in `DateTime`.

use arduino_hal::port::mode::Output;
use arduino_hal::port::{mode, Pin, PinOps};
use crate::std::extensions::binary_to_decimal::BcdExt;
use crate::std::extensions::str_to_unumber::StrToNumberExt;
use embedded_hal::digital::{InputPin, OutputPin};

#[derive(Copy, Clone, Default)]
/// Decimal calendar/time payload.
pub struct DateTime {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub day: u8,         // day of month: 1..31
    pub day_in_week: u8, // weekday: 1..7
    pub month: u8,
    pub year: u8,        // 00..99
}

/// DS1302 handle with runtime DAT direction switching.
pub struct RealTimeDS1302<CLK, DAT, RST> {
    clk: Pin<mode::Output, CLK>,
    rst: Pin<mode::Output, RST>,

    dat_out: Option<Pin<mode::Output, DAT>>,
    dat_in: Option<Pin<mode::Input<mode::Floating>, DAT>>,
}

// DS1302 command/register bytes
const REG_SECONDS: u8 = 0x80;
const REG_MINUTES: u8 = 0x82;
const REG_HOURS: u8 = 0x84;
const REG_DAY: u8 = 0x86; // day of month
const REG_MONTH: u8 = 0x88;
const REG_DAY_WEEK: u8 = 0x8A; // day of week
const REG_YEAR: u8 = 0x8C;
const REG_CONTROL: u8 = 0x8E;

// Masks
const SECONDS_MASK: u8 = 0x7F; // bit7 = CH
const MINUTES_MASK: u8 = 0x7F;
const HOURS_MASK: u8 = 0x3F; // 24-hour mode
const DAY_MASK: u8 = 0x3F; // 1..31
const MONTH_MASK: u8 = 0x1F; // 1..12
const YEAR_MASK: u8 = 0xFF; // 00..99
const DAY_IN_WEEK_MASK: u8 = 0x07; // 1..7

const CONTROL_WP_BIT: u8 = 0x80;

impl<CLK, DAT, RST> RealTimeDS1302<CLK, DAT, RST>
where
    CLK: PinOps,
    DAT: PinOps,
    RST: PinOps,
{
    /// Create DS1302 and put lines into idle-low state.
    pub fn new(mut clk: Pin<Output, CLK>, mut dat: Pin<Output, DAT>, mut rst: Pin<Output, RST>) -> Self {
        let _ = clk.set_low();
        let _ = rst.set_low();
        let _ = dat.set_low();
        Self {
            clk,
            rst,
            dat_out: Some(dat),
            dat_in: None,
        }
    }

    /// Control write protection (`enable = true` allows writes).
    pub fn enable_write(&mut self, enable: bool) {
        self.write_reg(REG_CONTROL, if enable { 0x00 } else { CONTROL_WP_BIT });
    }

    /// Write complete date/time.
    pub fn set_time(&mut self, dt: DateTime) {
        self.enable_write(true);

        self.write_reg(REG_SECONDS, dt.sec.dec_to_bdc() & SECONDS_MASK);
        self.write_reg(REG_MINUTES, dt.min.dec_to_bdc() & MINUTES_MASK);
        self.write_reg(REG_HOURS, dt.hour.dec_to_bdc() & HOURS_MASK);
        self.write_reg(REG_DAY, dt.day.dec_to_bdc() & DAY_MASK);
        self.write_reg(REG_MONTH, dt.month.dec_to_bdc() & MONTH_MASK);
        self.write_reg(REG_DAY_WEEK, dt.day_in_week.dec_to_bdc() & DAY_IN_WEEK_MASK);
        self.write_reg(REG_YEAR, dt.year.dec_to_bdc() & YEAR_MASK);

        self.enable_write(false);
    }

    /// Read complete date/time.
    pub fn read_time(&mut self) -> DateTime {
        let sec = (self.read_reg(REG_SECONDS) & SECONDS_MASK).bdc_to_dec();
        let min = (self.read_reg(REG_MINUTES) & MINUTES_MASK).bdc_to_dec();
        let hour = (self.read_reg(REG_HOURS) & HOURS_MASK).bdc_to_dec();
        let day = (self.read_reg(REG_DAY) & DAY_MASK).bdc_to_dec();
        let month = (self.read_reg(REG_MONTH) & MONTH_MASK).bdc_to_dec();
        let day_in_week = (self.read_reg(REG_DAY_WEEK) & DAY_IN_WEEK_MASK).bdc_to_dec();
        let year = (self.read_reg(REG_YEAR) & YEAR_MASK).bdc_to_dec();

        DateTime {
            sec,
            min,
            hour,
            day,
            day_in_week,
            month,
            year,
        }
    }

    fn write_reg(&mut self, reg: u8, value: u8) {
        self.dat_to_output();
        let _ = self.clk.set_low();
        let _ = self.rst.set_high();
        arduino_hal::delay_us(1);

        self.write_byte(reg);
        self.write_byte(value);

        let _ = self.rst.set_low();
        arduino_hal::delay_us(1);
    }

    fn write_read_command(&mut self, mut cmd: u8) {
        self.dat_to_output();

        for bit_index in 0..8 {
            {
                let dat = self.dat_out.as_mut().unwrap();

                if (cmd & 0x01) != 0 {
                    let _ = dat.set_high();
                } else {
                    let _ = dat.set_low();
                }
            }

            arduino_hal::delay_us(2);
            let _ = self.clk.set_high();
            arduino_hal::delay_us(2);

            // Release DAT before the last falling edge so chip can drive reply data.
            if bit_index == 7 {
                self.dat_to_input();
            }

            let _ = self.clk.set_low();
            arduino_hal::delay_us(2);

            cmd >>= 1;
        }
    }

    pub(crate) fn read_reg(&mut self, reg: u8) -> u8 {
        let _ = self.clk.set_low();
        let _ = self.rst.set_high();
        arduino_hal::delay_us(2);

        self.write_read_command(reg | 0x01);
        let v = self.read_byte();

        let _ = self.rst.set_low();
        arduino_hal::delay_us(2);

        v
    }

    fn write_byte(&mut self, mut value: u8) {
        self.dat_to_output();

        for _ in 0..8 {
            let dat = self.dat_out.as_mut().unwrap();

            if (value & 0x01) != 0 {
                let _ = dat.set_high();
            } else {
                let _ = dat.set_low();
            }

            let _ = self.clk.set_high();
            arduino_hal::delay_us(1);
            let _ = self.clk.set_low();
            arduino_hal::delay_us(1);

            value >>= 1;
        }
    }

    fn read_byte(&mut self) -> u8 {
        let mut value = 0u8;

        for i in 0..8 {
            let dat = self.dat_in.as_ref().unwrap();

            // First bit is already valid after read command phase.
            if dat.is_high() {
                value |= 1 << i;
            }

            if i != 7 {
                let _ = self.clk.set_high();
                arduino_hal::delay_us(2);
                let _ = self.clk.set_low();
                arduino_hal::delay_us(2);
            }
        }

        value
    }

    fn dat_to_output(&mut self) {
        if self.dat_out.is_none() {
            let pin = self.dat_in.take().unwrap().into_output();
            self.dat_out = Some(pin);
        }
    }

    fn dat_to_input(&mut self) {
        if self.dat_in.is_none() {
            let pin = self.dat_out.take().unwrap().into_floating_input();
            self.dat_in = Some(pin);
        }
    }

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
}
