use std::ops::{Add, Mul, Sub};

pub fn lerp<T, U>(a: T, b: T, weight: U) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<U, Output = T> + Copy,
{
    (b - a) * weight + a
}

pub fn smoothstep<T, U>(a: T, b: T, weight: U) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<U, Output = T> + Copy,
    U: Mul<f32, Output = U> + Copy,
    f32: Sub<U, Output = U> + Copy,
{
    (b - a) * (3.0 - weight * 2.0) * weight * weight + a
}

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
