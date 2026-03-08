//! HD44780 command bytes used by the LCD1602 driver.
//!
//! The values are standard HD44780 instructions.
//!
//! Note: `Test` are project helper for init sequencing:
//! - `Test = 0x03` is used as the **nibble** `0x03` during the classic 4-bit init ritual.
//!   It is **not** an actual HD44780 command.


#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
/// HD44780 instruction codes used by the driver.
///
/// Values map directly to the controller command byte.
/// See the module-level docs for notes about `Test`.

pub enum LcdCmd {
    // --- basic ---
    ClearDisplay = 0x01,
    ReturnHome = 0x02,

    EntryModeIncNoShift = 0x06, // cursor ++, no display shift

    DisplayOff = 0x08,
    DisplayOn = 0x0C,       // on, cursor off, blink off
    DisplayOnCursor = 0x0E,     // on, cursor on, blink off
    DisplayOnBlink = 0x0F,     // on, cursor on, blink on

    // --- function set (4-bit, 2 lines, 5x8 dots) ---
    FunctionSet4Bit2Line5x8 = 0x28,

    Test = 0x03,
}