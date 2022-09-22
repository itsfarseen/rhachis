//! Random generation functions convenient for game development.

use std::{
    ops::{Bound, RangeBounds},
    time::SystemTime,
};

use glam::Vec2;

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
        let mut a = x.rotate_right(13).wrapping_pow(x);
        a = a.wrapping_mul(a.wrapping_mul(self.seed));
        a = a.wrapping_add(19990303);
        a = a.wrapping_mul(a.rotate_left(8));
        a = a.wrapping_pow(self.seed.wrapping_pow(x));
        a = a.wrapping_add(1376312589);
        a = a.wrapping_pow(4);
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

        let x_noise = noise.get(grid_pos.x * 2) as f32 - u32::MAX as f32 / 2.0;
        let y_noise = noise.get(grid_pos.y * 2 + 1) as f32 - u32::MAX as f32 / 2.0;

        Vec2::new(x_noise, y_noise).normalize()
    }

    let grid_pos = pos.floor();
    let offset_pos = pos - grid_pos;

    let gradient_top_left = get_gradient(noise, grid_pos);
    let gradient_top_right = get_gradient(noise, grid_pos + Vec2::new(1.0, 0.0));
    let gradient_bottom_left = get_gradient(noise, grid_pos + Vec2::new(0.0, 1.0));
    let gradient_bottom_right = get_gradient(noise, grid_pos + Vec2::new(1.0, 1.0));

    let difference_top_left = pos - grid_pos;
    let difference_top_right = pos - (grid_pos + Vec2::new(1.0, 0.0));
    let difference_bottom_left = pos - (grid_pos + Vec2::new(0.0, 1.0));
    let difference_bottom_right = pos - (grid_pos + Vec2::new(1.0, 1.0));

    let influence_top_left = gradient_top_left.dot(difference_top_left);
    let influence_top_right = gradient_top_right.dot(difference_top_right);
    let influence_bottom_left = gradient_bottom_left.dot(difference_bottom_left);
    let influence_bottom_right = gradient_bottom_right.dot(difference_bottom_right);

    interpolate(
        interpolate(influence_top_left, influence_top_right, offset_pos.x),
        interpolate(influence_bottom_left, influence_bottom_right, offset_pos.x),
        offset_pos.y,
    )
}
