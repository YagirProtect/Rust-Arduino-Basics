//! Simple heartbeat LED helper.
//!
//! Toggles an output pin every `blink_time` milliseconds.
//! Useful for quick liveness indication in the main loop.

use arduino_hal::port::{mode, Pin, PinOps};

/// Periodic LED blinker.
pub struct HeartbeatDiode<PW> {
    power_pin: Pin<mode::Output, PW>,
    time: u32,
    blink_time: u32,
    state: bool,
}

impl<PW> HeartbeatDiode<PW>
where
    PW: PinOps,
{
    /// Create heartbeat blinker for a digital output pin.
    pub fn new(power_pin: Pin<mode::Output, PW>, blink_time: u32) -> Self {
        Self {
            power_pin,
            time: 0,
            blink_time,
            state: false,
        }
    }

    /// Update LED state based on current timestamp (ms).
    pub fn update(&mut self, time: u32) {
        if (time.wrapping_sub(self.time) >= self.blink_time) {
            self.time = time;
            self.state = !self.state;

            if (self.state == false) {
                self.power_pin.set_low();
            } else {
                self.power_pin.set_high();
            }
        }
    }
}
