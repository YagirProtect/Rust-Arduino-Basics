#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
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

    Init = 0x00
}