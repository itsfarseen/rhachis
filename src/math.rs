use std::ops::{Sub, Mul, Add};

pub fn lerp<T, U>(a: T, b: T, weight: U) -> T
where T: Add<T, Output = T> + Sub<T, Output = T> + Mul<U, Output = T> {
    (b - a) * weight + a
}
