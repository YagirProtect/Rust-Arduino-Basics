//! Millisecond timebase for ATmega328P using Timer/Counter0 (CTC + Compare A interrupt).
//!
//! This module configures **Timer0** in **CTC** mode to generate an interrupt every 1 ms.
//! The ISR increments a global millisecond counter. The counter is monotonic and wraps
//! around on overflow (u32 wraps roughly every 49.7 days).
//!
//! ## Notes
//! - Designed for **ATmega328P @ 16 MHz** with prescaler **64** and `OCR0A=249`.
//! - Uses an interrupt, so global interrupts must be enabled after initialization.
//! - Timer0 is commonly used by Arduino core for `millis()/delay()`; if you rely on those,
//!   do not reconfigure Timer0 in the same firmware.
//!
//! ## Overflow handling
//! The counter is `u32` and wraps naturally. Use `wrapping_sub` for time deltas:
//! `now.wrapping_sub(last) >= dt`.


use core::cell::Cell;
use avr_device::interrupt::Mutex;
static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS.borrow(cs).get())
}
/// Global millisecond timer backed by **Timer0 Compare A** interrupt.
///
/// `GlobalTimer` is a lightweight handle providing access to a global millisecond counter.
/// The underlying counter is stored in a global static and updated from the ISR.
///
/// This type does **not** own hardware resources at runtime; it only performs
/// initialization in [`GlobalTimer::new`] and then can be copied/held as a handle.
///
/// # Example
/// ```ignore
/// let dp = arduino_hal::Peripherals::take().unwrap();
/// let timer = GlobalTimer::new(&dp.TC0);
/// unsafe { avr_device::interrupt::enable(); }
///
/// let mut last = timer.millis();
/// loop {
///     let now = timer.millis();
///     if now.wrapping_sub(last) >= 200 {
///         last = now;
///         // do something periodically
///     }
/// }
/// ```
///
/// # Notes
/// - The returned value wraps around (`u32`).
/// - Requires interrupts to be enabled to advance.
pub struct GlobalTimer{}

impl GlobalTimer {
    /// Initializes **Timer0** to generate a 1 ms tick (CTC + Compare A interrupt).
    ///
    /// This configures:
    /// - CTC mode (`WGM0 = CTC`)
    /// - Compare value `OCR0A = 249` (for 16 MHz + prescaler 64)
    /// - Prescaler = 64
    /// - Enables `TIMER0_COMPA` interrupt (`OCIE0A = 1`)
    ///
    /// # Safety
    /// Safe to call once at startup. Reconfiguring Timer0 while other code depends on it
    /// may break PWM/other timing functionality.
    pub fn new(tc0: &arduino_hal::pac::TC0) -> Self {
        const OCR0A_1MS: u8 = 249;

        tc0.tccr0a().write(|w| w.wgm0().ctc());
        tc0.ocr0a().write(|w| unsafe { w.bits(OCR0A_1MS) });
        tc0.tccr0b().write(|w| w.cs0().prescale_64());
        tc0.timsk0().write(|w| w.ocie0a().set_bit());


        Self{}
    }

    /// Returns the number of milliseconds since the timer was initialized.
    ///
    /// The counter increments in the `TIMER0_COMPA` ISR and wraps around on overflow.
    /// Use `wrapping_sub` when computing elapsed time.
    ///
    /// If global interrupts are disabled, this value will stop increasing.
    pub fn millis(&self) -> u32 {
        return millis();
    }
}


//INTERRUPTS


/// Timer0 Compare Match A ISR.
///
/// Increments the global millisecond counter.
#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let c = MILLIS.borrow(cs);
        c.set(c.get().wrapping_add(1));
    });
}
