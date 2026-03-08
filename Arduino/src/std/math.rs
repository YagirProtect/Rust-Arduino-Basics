//! Small math helpers (no_std).
//!
//! Provides:
//! - [`inverse_lerp`] — map value `v` from `[a,b]` to `t` in `[0,1]` (not clamped)
//! - [`lerp`] — linear interpolation between `a` and `b` by `t`
//! - [`normalize`] — convenience mapping from `[a,b]` to `[-1,1]`
//!
//! Useful for joystick normalization and simple sensor scaling.


/// Map `v` from range `[a,b]` to `t` in `[0,1]` (not clamped).
pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}

/// Linear interpolation between `a` and `b` by `t`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Map `v` from range `[a,b]` to `[-1,1]` (not clamped).
pub fn normalize(a: f32, b:f32, v: f32) -> f32 {
    let t = inverse_lerp(a, b, v); // 0..1
    lerp(-1.0, 1.0, t)                           // -1..1
}
