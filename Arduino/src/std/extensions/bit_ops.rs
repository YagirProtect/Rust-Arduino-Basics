//! Bit-level helpers for integer register work.
//!
//! Useful for sensor/RTC/GPIO registers where single-bit flags are common.

/// Extension trait with safe bit operations for integer types.
pub trait BitOpsExt: Sized + Copy {
    /// Number of bits in the type.
    const BITS: u8;

    /// Return `true` when bit at `index` is set.
    ///
    /// Out-of-range index returns `false`.
    fn bit_is_set(self, index: u8) -> bool;

    /// Return value with bit at `index` set to `1`.
    ///
    /// Out-of-range index returns unchanged value.
    fn set_bit(self, index: u8) -> Self;

    /// Return value with bit at `index` cleared to `0`.
    ///
    /// Out-of-range index returns unchanged value.
    fn clear_bit(self, index: u8) -> Self;

    /// Return value with bit at `index` inverted.
    ///
    /// Out-of-range index returns unchanged value.
    fn toggle_bit(self, index: u8) -> Self;

    /// Return value with bit at `index` forced to `state`.
    ///
    /// Out-of-range index returns unchanged value.
    fn write_bit(self, index: u8, state: bool) -> Self;
}

macro_rules! impl_bit_ops_for_uint {
    ($t:ty, $bits:expr) => {
        impl BitOpsExt for $t {
            const BITS: u8 = $bits;

            fn bit_is_set(self, index: u8) -> bool {
                if index >= <Self as BitOpsExt>::BITS {
                    return false;
                }
                let mask = (1 as $t).checked_shl(index as u32).unwrap_or(0);
                (self & mask) != 0
            }

            fn set_bit(self, index: u8) -> Self {
                if index >= <Self as BitOpsExt>::BITS {
                    return self;
                }
                let mask = (1 as $t).checked_shl(index as u32).unwrap_or(0);
                self | mask
            }

            fn clear_bit(self, index: u8) -> Self {
                if index >= <Self as BitOpsExt>::BITS {
                    return self;
                }
                let mask = (1 as $t).checked_shl(index as u32).unwrap_or(0);
                self & !mask
            }

            fn toggle_bit(self, index: u8) -> Self {
                if index >= <Self as BitOpsExt>::BITS {
                    return self;
                }
                let mask = (1 as $t).checked_shl(index as u32).unwrap_or(0);
                self ^ mask
            }

            fn write_bit(self, index: u8, state: bool) -> Self {
                if state {
                    self.set_bit(index)
                } else {
                    self.clear_bit(index)
                }
            }
        }
    };
}

impl_bit_ops_for_uint!(u8, 8);
impl_bit_ops_for_uint!(u16, 16);
impl_bit_ops_for_uint!(u32, 32);
