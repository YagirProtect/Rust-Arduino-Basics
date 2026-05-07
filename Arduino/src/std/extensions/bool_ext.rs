//! Small helpers for boolean values.

/// Extension trait for convenient `bool` conversions and toggling.
pub trait BoolExt {
    /// Convert `bool` to `u8` (`true => 1`, `false => 0`).
    fn as_u8(self) -> u8;

    /// Convert `bool` to `u16` (`true => 1`, `false => 0`).
    fn as_u16(self) -> u16;

    /// Invert the value in place.
    fn toggle(&mut self);
}

impl BoolExt for bool {
    fn as_u8(self) -> u8 {
        if self { 1 } else { 0 }
    }

    fn as_u16(self) -> u16 {
        if self { 1 } else { 0 }
    }

    fn toggle(&mut self) {
        *self = !*self;
    }
}
