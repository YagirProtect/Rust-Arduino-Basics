//! Small project utilities ("std" but for no_std).
//!
//! This folder holds tiny helpers commonly used across modules:
//! - [`global_timer`] — a millisecond timebase using Timer0 ISR
//! - [`io`] — serial helper with a heapless formatting buffer
//! - [`math`] — simple lerp/normalize helpers
//! - [`std`] — misc functions (e.g. enable interrupts)
//!
//! These are not Rust's `std`; the name is project-local.


pub mod global_timer;
pub mod std;
pub mod io;
pub mod math;
pub mod extensions;