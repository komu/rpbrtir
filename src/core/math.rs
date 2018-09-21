use core::types::{Float, PI};

#[inline]
pub fn solve_quadratic(a: Float, b: Float, c: Float) -> Option<(Float, Float)> {
    let discrim = b * b - 4.0 * a * c;
    if discrim < 0.0 {
        return None;
    }

    let q = if b < 0.0 {
        -0.5 * (b - discrim.sqrt())
    } else {
        -0.5 * (b + discrim.sqrt())
    };

    let t0 = q / a;
    let t1 = c / q;

    Some((t0.min(t1), t0.max(t1)))
}

#[inline]
pub fn radians(degrees: Float) -> Float {
    (PI / 180.0) * degrees
}

#[inline]
pub fn clamp(val: Float, low: Float, high: Float) -> Float {
    val.max(low).min(high)
}

#[inline]
pub fn floor_to_int(val: Float) -> i32 {
    val.floor() as i32
}

#[inline]
pub fn lerp(t: Float, v1: Float, v2: Float) -> Float {
    (1.0 - t) * v1 + t * v2
}

