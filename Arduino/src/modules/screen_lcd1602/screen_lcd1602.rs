//! LCD1602 driver (HD44780) over an I²C backpack (PCF8574).
//!
//! The LCD itself is a parallel device. The backpack exposes 8 GPIO pins over I²C and is
//! typically wired like this (most common mapping):
//! - P0 → RS
//! - P1 → RW
//! - P2 → E
//! - P3 → Backlight
//! - P4 → D4
//! - P5 → D5
//! - P6 → D6
//! - P7 → D7
//!
//! In 4-bit mode each byte is sent as two nibbles (high then low). Each nibble is "latched"
//! by pulsing `E` (Enable) high then low.
//!
//! ## Modes
//! - [`EMode::Linear`]: blocking calls; init performs `delay_ms(...)` waits.
//! - [`EMode::Async`]: commands/data are enqueued and executed via [`ScreenLCD1602::update`].
//!
//! ## Internal line buffer
//! The driver uses a `heapless::String<64>` as a formatting scratchpad. In the current
//! implementation `get_line()` clears the buffer before returning it.
//!
//! ## Newlines
//! `print()` interprets `\n` as move to the second row (`row=1`) and `\r` as ignored.


use crate::modules::screen_lcd1602::screen_alcd1602_async::{Op, OpQueue};
use crate::modules::screen_lcd1602::screen_lcd1602::EMode::Linear;
use crate::modules::screen_lcd1602::screen_lcd1602cmd::LcdCmd;
use core::cmp::PartialEq;
use embedded_hal::i2c::I2c;
use heapless::String;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// LCD driver mode.
///
/// - `Linear`: blocking operations (simple).
/// - `Async`: queue operations and call [`ScreenLCD1602::update`] to progress.

pub enum EMode {
    Linear,
    Async,
}

/// LCD1602 driver handle.
///
/// Owns the I²C address and internal state (backlight, mode, optional queue and line buffer).
/// Use [`ScreenLCD1602::get_line`] + `core::fmt::Write` to format text without allocation.

pub struct ScreenLCD1602 {
    addr: u8,
    backlight: bool,
    mode: EMode,

    line: String<64>,

    // async runtime
    q: OpQueue,
    next_ms: u32,

    display_on: bool,
}

impl ScreenLCD1602 {
    /// Create and initialize the LCD.
    ///
    /// Performs the standard HD44780 4-bit init ritual (`0x03`×3, then `0x02`),
    /// followed by function set / display off / clear / entry mode / display on.

    pub fn new(addr: u8, i2c: &mut arduino_hal::I2c, mode: EMode) -> Self {
        let mut screen = Self {
            addr,
            backlight: true,
            mode,
            q: OpQueue::new(),
            next_ms: 0,

            line: String::new(),
            display_on: true
        };

        arduino_hal::delay_ms(50);

        screen.write4(i2c, LcdCmd::Test as u8, false);
        arduino_hal::delay_ms(5);
        screen.write4(i2c, LcdCmd::Test as u8, false);
        arduino_hal::delay_ms(5);
        screen.write4(i2c, LcdCmd::Test as u8, false);
        arduino_hal::delay_ms(1);

        screen.write4(i2c, LcdCmd::ReturnHome as u8, false);
        arduino_hal::delay_ms(1);

        screen.command_blocking(i2c, LcdCmd::FunctionSet4Bit2Line5x8 as u8);
        screen.command_blocking(i2c, LcdCmd::DisplayOff as u8);

        screen.command_blocking(i2c, LcdCmd::ClearDisplay as u8);
        arduino_hal::delay_ms(2);

        screen.command_blocking(i2c, LcdCmd::EntryModeIncNoShift as u8);
        screen.command_blocking(i2c, LcdCmd::DisplayOn as u8);

        screen
    }
    /// Progress queued operations in async mode.
    ///
    /// Call this from your main loop. In `Linear` mode this is a no-op.

    pub fn update(&mut self, now_ms: u32, i2c: &mut arduino_hal::I2c) {
        if self.mode != EMode::Async {
            return;
        }
        if now_ms < self.next_ms {
            return;
        }

        let op = match self.q.pop() {
            Some(op) => op,
            None => return,
        };

        match op {
            Op::WaitMs(ms) => {
                self.next_ms = now_ms.saturating_add(ms as u32);
            }
            Op::Cmd(cmd) => {
                self.command_blocking(i2c, cmd);
                if cmd == LcdCmd::ClearDisplay as u8 || cmd == LcdCmd::ReturnHome as u8 {
                    // эти команды реально долгие (~1.5ms)
                    self.next_ms = now_ms.saturating_add(2);
                }
            }
            Op::Data(b) => {
                self.send_byte(i2c, b, true);
            }
        }
    }

    /// Send a command (blocking in Linear, queued in Async).
    pub fn command(&mut self, i2c: &mut arduino_hal::I2c, cmd: LcdCmd) {
        match self.mode {
            EMode::Linear => self.command_blocking(i2c, cmd as u8),
            EMode::Async => {
                let _ = self.q.push(Op::Cmd(cmd as u8));
                if cmd == LcdCmd::ClearDisplay || cmd == LcdCmd::ReturnHome {
                    let _ = self.q.push(Op::WaitMs(2));
                }
            }
        }
    }

    /// Get a mutable handle to the internal formatting buffer.
    ///
    /// Current behavior: clears the buffer before returning it.

    pub fn get_line(&mut self) -> &mut String<64>{
        self.line.clear();
        return &mut self.line
    }

    /// Clear the display (`0x01`).
    pub fn clear(&mut self, i2c: &mut arduino_hal::I2c) {
        self.command(i2c, LcdCmd::ClearDisplay);
    }

    /// Turn the display off.
    pub fn display_off(&mut self, i2c: &mut arduino_hal::I2c, is_backlight: bool) {
        self.backlight = is_backlight;
        self.command(i2c, LcdCmd::DisplayOff);
        self.display_on = false;
    }

    /// Turn the display on.
    pub fn display_on(&mut self, i2c: &mut arduino_hal::I2c) {
        self.backlight = true;
        self.command(i2c, LcdCmd::DisplayOn);
        self.display_on = true;
    }

    /// Set cursor to point.
    pub fn set_cursor(&mut self, i2c: &mut arduino_hal::I2c, col: u8, row: u8) {
        let row_offsets = [0x00u8, 0x40u8, 0x14u8, 0x54u8];
        let r = if (row as usize) < row_offsets.len() {
            row as usize
        } else {
            0
        };
        let cmd = 0x80 | (col + row_offsets[r]);

        match self.mode {
            EMode::Linear => self.command_blocking(i2c, cmd),
            EMode::Async => {
                let _ = self.q.push(Op::Cmd(cmd));
            }
        }
    }

    /// Print the current line buffer to the LCD.
    ///
    /// Interprets `\n` as move to row 1 and ignores `\r`.
    /// In async mode, bytes are enqueued (requires [`ScreenLCD1602::update`]).

    pub fn print(&mut self, i2c: &mut arduino_hal::I2c) {

        self.clear(i2c);

        let mut col: u8 = 0;
        let mut row: u8 = 0;

        let s = self.line.clone();

        self.set_cursor(i2c, 0, 0);

        for &b in s.as_bytes() {
            match b {
                b'\n' => {
                    row = (row + 1).min(1); // 1602: 0/1
                    col = 0;
                    self.set_cursor(i2c, col, row);
                }
                b'\r' => {
                    // ignore
                }
                _ => {
                    if (self.mode == Linear) {
                        self.send_byte(i2c, b, true);
                    }else{
                        let _ = self.q.push(Op::Data(b));
                    }

                    col += 1;
                    if col >= 16 {
                        if row == 0 {
                            row = 1;
                            col = 0;
                            self.set_cursor(i2c, col, row);
                        } else {

                        }
                    }
                }
            }
        }
    }

    // -------------------- LOW-LEVEL --------------------

    fn command_blocking(&self, i2c: &mut arduino_hal::I2c, cmd: u8) {
        self.send_byte(i2c, cmd, false);
        if cmd == LcdCmd::ClearDisplay as u8 || cmd == LcdCmd::ReturnHome as u8 {
            arduino_hal::delay_ms(2);
        }
    }

    fn send_byte(&self, i2c: &mut arduino_hal::I2c, b: u8, rs: bool) {
        let hi = (b >> 4) & 0x0F;
        let lo = b & 0x0F;
        self.write4(i2c, hi, rs);
        self.write4(i2c, lo, rs);
    }

    fn write4(&self, i2c: &mut arduino_hal::I2c, nibble: u8, rs: bool) {
        let data = self.generate_command(nibble, rs, false);
        self.expander_write(i2c, data);
        self.pulse_enable(i2c, data);
    }

    fn expander_write(&self, i2c: &mut arduino_hal::I2c, data: u8) {
        let _ = i2c.write(self.addr, &[data]);
    }

    fn pulse_enable(&self, i2c: &mut arduino_hal::I2c, data: u8) {
        // E high then low
        self.expander_write(i2c, data | (1 << 2)); // E=1 (P2)
        arduino_hal::delay_us(1);
        self.expander_write(i2c, data & !(1 << 2)); // E=0
        arduino_hal::delay_us(50);
    }

    fn generate_command(&self, nibble: u8, rs: bool, rw: bool) -> u8 {
        // P4..P7 = D4..D7
        let mut v = (nibble & 0x0F) << 4;

        if rs { v |= 1 << 0; }          // P0 = RS
        if rw { v |= 1 << 1; }          // P1 = RW
        if self.backlight { v |= 1 << 3; } // P3 = Backlight

        v
    }

    pub fn is_display_on(&self) -> bool { self.display_on }
}


//Mode: Linear
// #[arduino_hal::entry]
// fn main() -> ! {
//     let dp = arduino_hal::Peripherals::take().unwrap();
//     let pins = arduino_hal::pins!(dp);
//
//
//     let mut i2c = arduino_hal::I2c::new(
//         dp.TWI,
//         pins.a4.into_pull_up_input(),
//         pins.a5.into_pull_up_input(),
//         100_000,
//     );
//     let mut screen = ScreenLCD1602::new(0x27, &mut i2c, EMode::Linear);
//
//     let mut val = 1337;
//     write!(screen.get_line(), "{}", val).unwrap();
//     screen.print(&mut i2c);
//     loop {
//     }
// }




//Mode: Async
// #[arduino_hal::entry]
// fn main() -> ! {
//     let dp = arduino_hal::Peripherals::take().unwrap();
//     let pins = arduino_hal::pins!(dp);
//
//
//     let mut i2c = arduino_hal::I2c::new(
//         dp.TWI,
//         pins.a4.into_pull_up_input(),
//         pins.a5.into_pull_up_input(),
//         100_000,
//     );
//
//     enable_interrupts();
//     let timer = GlobalTimer::new(&dp.TC0);
//
//
//     let mut screen = ScreenLCD1602::new(0x27, &mut i2c, EMode::Async);
//
//     let mut last = timer.millis();
//     let mut val = 0;
//     loop {
//         let now = timer.millis();
//
//         val += 1;
//
//
//         if (val == 0){
//             screen.display_off(&mut i2c);
//         }
//         if (val == 10000) {
//             write!(screen.get_line(), "nu ti i lox").unwrap();
//             screen.display_on(&mut i2c);
//             screen.print(&mut i2c);
//         }
//         if (val == 100000){
//             screen.clear(&mut i2c);
//         }
//         if (val > 150000){
//             val = 0;
//         }
//
//         screen.update(now, &mut i2c);
//
//         last = now;
//     }
// }