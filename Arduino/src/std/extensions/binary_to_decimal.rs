//! BCD/decimal conversion helpers for `u8`.
//!
//! Used by RTC modules (for example DS1302) where register values are stored in BCD.

/// Extension trait for converting between BCD and decimal.
pub trait BcdExt {
    /// Convert BCD byte (`0x42`) into decimal (`42`).
    ///
    /// Note: method name keeps existing project spelling (`bdc_to_dec`).
    fn bdc_to_dec(self) -> u8;

    /// Convert decimal value (`42`) into BCD byte (`0x42`).
    fn dec_to_bdc(self) -> u8;
}

impl BcdExt for u8 {
    fn bdc_to_dec(self) -> u8 {
        ((self >> 4) * 10) + (self & 0x0F)
    }

    fn dec_to_bdc(self) -> u8 {
        ((self / 10) << 4) | (self % 10)
    }
}
