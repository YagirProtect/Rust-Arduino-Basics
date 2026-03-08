//! Misc helper functions.
//!
//! Currently contains a helper to enable global interrupts.


/// Enable global interrupts.
///
/// On AVR this sets the I-bit in SREG. Must be called after configuring interrupt sources.

pub fn enable_interrupts() {
    unsafe { avr_device::interrupt::enable(); }
}