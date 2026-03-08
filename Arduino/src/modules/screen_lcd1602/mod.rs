//! LCD1602 (HD44780) over I²C backpack (PCF8574).
//!
//! This submodule contains:
//! - command definitions (`screen_lcd1602cmd`)
//! - a tiny async ring queue (`screen_alcd1602_async`)
//! - the main driver (`screen_lcd1602`)
//!
//! The driver supports both blocking and queued (async-like) usage.


pub mod screen_lcd1602;
mod screen_lcd1602cmd;
mod screen_alcd1602_async;