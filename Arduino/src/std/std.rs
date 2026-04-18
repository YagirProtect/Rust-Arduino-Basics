//! Misc helper functions.
//!
//! Currently contains a helper to enable global interrupts.


/// Enable global interrupts.
///
/// On AVR this sets the I-bit in SREG. Must be called after configuring interrupt sources.

pub fn enable_interrupts() {
    unsafe { avr_device::interrupt::enable(); }
}


fn parse_u8(s: &str) -> u8 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut n = 0u8;
    while i < bytes.len() {
        n = n * 10 + (bytes[i] - b'0');
        i += 1;
    }
    n
}
