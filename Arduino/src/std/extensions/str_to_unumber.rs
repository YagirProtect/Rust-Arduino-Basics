//! Lightweight digit-only string to unsigned number conversion.
//!
//! Intended for tiny `no_std` use-cases (for example compile-time date/time env strings).
//! Non-digit characters are ignored.

/// Extension trait for parsing decimal digits from `&str`.
pub trait StrToNumberExt {
    /// Parse all digits from string into `u8`.
    ///
    /// Non-digit bytes are skipped. Overflow behavior follows `u8` arithmetic.
    fn to_u8(self) -> u8;

    /// Parse all digits from string into `u16`.
    ///
    /// Non-digit bytes are skipped. Overflow behavior follows `u16` arithmetic.
    fn to_u16(self) -> u16;
}

impl StrToNumberExt for &str {
    fn to_u8(self) -> u8 {
        let bytes = self.as_bytes();
        let mut i = 0;
        let mut n = 0u8;

        while i < bytes.len() {
            let b = bytes[i];
            if b >= b'0' && b <= b'9' {
                n = n * 10 + (b - b'0');
            }
            i += 1;
        }

        n
    }

    fn to_u16(self) -> u16 {
        let bytes = self.as_bytes();
        let mut i = 0;
        let mut n = 0u16;

        while i < bytes.len() {
            let b = bytes[i];
            if b >= b'0' && b <= b'9' {
                n = n * 10 + (b - b'0') as u16;
            }
            i += 1;
        }

        n
    }
}
