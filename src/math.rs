//! A collection of functions that are useful for games.

use std::ops::{Add, Mul, Sub};

/// An implementation of linear interpolation.
pub fn lerp<T, U>(a: T, b: T, weight: U) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<U, Output = T> + Copy,
{
    (b - a) * weight + a
}

#[test]
fn lerp_test() {
    assert_eq!(lerp(0.0, 1.0, 0.5), 0.5);
    assert_eq!(lerp(0.5, 1.0, 0.5), 0.75);
    assert_eq!(lerp(0.75, 100.0, 0.25), 25.5625);
}

/// An implementation of smoothstep interpolation.
pub fn smoothstep<T, U>(a: T, b: T, weight: U) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<U, Output = T> + Copy,
    U: Mul<f32, Output = U> + Copy,
    f32: Sub<U, Output = U> + Copy,
{
    (b - a) * (3.0 - weight * 2.0) * weight * weight + a
}

/// An implementation of smootherstep interpolation.
pub fn smootherstep<T, U>(a: T, b: T, weight: U) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<U, Output = T> + Copy,
    U: Add<f32, Output = U>
        + Mul<U, Output = U>
        + Mul<f32, Output = U>
        + Sub<f32, Output = U>
        + Copy,
    f32: Sub<U, Output = U> + Copy,
{
    (b - a) * ((weight * (weight * 6.0 - 15.0) + 10.0) * weight * weight * weight) + a
}
