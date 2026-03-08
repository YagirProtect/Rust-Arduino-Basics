//! Serial I/O helper for Arduino Uno/Nano (USART0).
//!
//! `IoUno` wraps `arduino_hal::Usart` and provides a small `heapless::String` scratch buffer.
//! The typical workflow is:
//! 1) build a log line into `io.str()` using `write!`/`writeln!`
//! 2) call [`IoUno::log`] to transmit it over serial.
//!
//! ## Why heapless
//! AVR targets are `no_std` and typically avoid heap allocation. `heapless::String` stores
//! the formatted text inline.


use arduino_hal::hal::port::{PD0, PD1};
use arduino_hal::hal::Usart;
use arduino_hal::port::mode::{Floating, Input, Output};
use arduino_hal::port::{Pin, D0, D1};
use arduino_hal::usart::Baudrate;
use arduino_hal::DefaultClock;
use avr_device::atmega328p::USART0;
use heapless::String;
use ufmt::uwriteln;

/// Serial helper for Arduino Uno/Nano (USART0) with a heapless formatting buffer.
///
/// Use [`IoUno::str`] to get a scratch string, format into it, then call [`IoUno::log`].

pub struct IoUno {
    serial: Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>, DefaultClock>,
    string: String<64>
}


impl IoUno {
    /// Create a new USART0 logger with given baud rate.
    pub fn new(usart0: USART0, rx: Pin<Input<Floating>, D0>, tx: Pin<Input<Floating>, D1>, val: i32) -> Self {

        let serial = arduino_hal::Usart::new(
            usart0,
            rx,
            tx.into_output(),
            Baudrate::new(val as u32),
        );

        Self{
            serial,
            string: String::new()
        }
    }

    /// Clear and return the scratch string buffer for formatting.
    pub fn str(&mut self) -> &mut String<64>{

        self.string.clear();
        return &mut self.string;
    }


    /// Get mutable access to the underlying USART driver.
    pub fn get_serial(&mut self) -> &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>, DefaultClock>{
        return &mut self.serial;
    }
    /// Send the current scratch buffer over serial (adds newline).
    pub fn log(&mut self) {
        let _ = uwriteln!(&mut self.serial, "{}", self.string.as_str());
    }
}