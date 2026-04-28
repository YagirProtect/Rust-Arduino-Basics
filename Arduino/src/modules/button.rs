//! Pull-up button helper.
//!
//! The button is expected to be wired as active-low:
//! press connects pin to GND, release leaves internal pull-up high.

use arduino_hal::port::mode::{Input, Output, PullUp};
use arduino_hal::port::{mode, Pin, PinOps};

/// Active-low button input wrapper.
pub struct Button<IN> {
    input_pin: Pin<mode::Input<PullUp>, IN>,
}

impl<IN> Button<IN>
where
    IN: PinOps,
{
    /// Create a new button from a pull-up configured input pin.
    pub fn new(input: Pin<mode::Input<PullUp>, IN>) -> Self {
        Self {
            input_pin: input,
        }
    }

    /// Returns `true` while button is physically pressed.
    pub fn is_pressed(&self) -> bool {
        self.input_pin.is_low()
    }
}
