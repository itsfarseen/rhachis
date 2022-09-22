//! Random generation functions convenient for game development.

use std::{
    f32::consts::TAU,
    ops::{Bound, RangeBounds},
    time::SystemTime,
};

use glam::Vec2;

use crate::math::lerp;

/// A random number generator that can produce consistent but still unpredictable outputs given
/// the same inputs. Useful for perlin noise generation.
pub struct Noise {
    /// The seed used to make generated numbers unique.
    pub seed: u32,
    /// The counter for `Noise::next` and `Noise::next_range`.
    pub index: u32,
}

impl Noise {
    /// Create a `Noise` instance. The seed is set to the number of seconds since the unix epoch.
    /// To avoid epochalypse problems, it actually returns `seconds % u32::MAX` so it will
    /// wrap around once it runs out of 32 bit numbers
    pub fn new() -> Noise {
        let seed = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % u32::MAX as u64) as u32;

        Self { seed, index: 0 }
    }

    /// Create a `Noise` instance with seed already decided.
    pub fn from_seed(seed: u32) -> Noise {
        Self { seed, index: 0 }
    }

    /// Get a pseudorandom number associated with the value `x`.
    pub fn get(&self, x: u32) -> u32 {
        let mut a = x.wrapping_mul(4294967291);
        a = a.wrapping_add(self.seed);
        a = a.wrapping_pow(5);
        a ^= a.rotate_right(5);
        a ^= a.rotate_left(9);
        a
    }

    #[allow(clippy::should_implement_trait)]
    /// Get the next pseudorandom number. This increments an internal counter
    /// and just returns the number calculated from the value of the counter.
    pub fn next(&mut self) -> u32 {
        self.index += 1;
        self.get(self.index)
    }

    /// Get a pseudorandom number in the range of `range`.
    pub fn get_range<T: RangeBounds<u32>>(&self, index: u32, range: T) -> u32 {
        let start = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + 1,
            Bound::Unbounded => u32::MIN,
        };
        let end = match range.end_bound() {
            Bound::Included(x) => *x + 1,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => u32::MAX,
        };
        self.get(index) % (end - start) + start
    }

    /// Get the next pseudorandom number in the range of `range`.
    pub fn next_range<T: RangeBounds<u32>>(&mut self, range: T) -> u32 {
        self.index += 1;
        self.get_range(self.index, range)
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the value of `pos` on a 2D perlin noise functions. The interpolation function
/// is left specified by the caller, but is typically one of the interpolation functions in
/// `math`.
///
/// ## Example:
/// ```
/// use rhachis::rand::{Noise, perlin_2d};
///
/// let noise = Noise::new();
/// let height = perlin_2d(&noise, (1.0, 1.0).into(), rhachis::math::lerp);
/// ```
pub fn perlin_2d<F: Fn(f32, f32, f32) -> f32>(noise: &Noise, pos: Vec2, interpolate: F) -> f32 {
    fn get_gradient(noise: &Noise, grid_pos: Vec2) -> Vec2 {
        let grid_pos = grid_pos.as_uvec2();

        let angle = TAU * 1000.0 / noise.get_range(noise.get(grid_pos.x) ^ grid_pos.y, 1..6283) as f32;

        Vec2::new(angle.sin(), angle.cos())
    }

    let corners = [
        pos.floor(),
        pos.floor() + Vec2::new(1.0, 0.0),
        pos.floor() + Vec2::new(0.0, 1.0),
        pos.floor() + Vec2::new(1.0, 1.0),
    ];

    let offsets = [
        corners[0] - pos,
        corners[1] - pos,
        corners[2] - pos,
        corners[3] - pos,
    ];

    let influence = [
        get_gradient(noise, corners[0]).dot(offsets[0]),
        get_gradient(noise, corners[1]).dot(offsets[1]),
        get_gradient(noise, corners[2]).dot(offsets[2]),
        get_gradient(noise, corners[3]).dot(offsets[3]),
    ];

    let offset = pos - pos.floor();
    interpolate(
        interpolate(influence[0], influence[1], offset.x),
        interpolate(influence[2], influence[3], offset.x),
        offset.y,
    )
}
