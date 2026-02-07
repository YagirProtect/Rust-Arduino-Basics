pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn normalize(a: f32, b:f32, v: f32) -> f32 {
    let t = inverse_lerp(a, b, v); // 0..1
    lerp(-1.0, 1.0, t)                           // -1..1
}
