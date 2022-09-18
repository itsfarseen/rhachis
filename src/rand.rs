use std::{
    ops::{Bound, RangeBounds},
    time::SystemTime,
};

use glam::Vec2;

use crate::math::lerp;

pub struct Noise {
    pub seed: u32,
    pub index: u32,
}

impl Noise {
    pub fn new() -> Noise {
        let seed = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % u32::MAX as u64) as u32;

        Self { seed, index: 0 }
    }

    pub fn from_seed(seed: u32) -> Noise {
        Self { seed, index: 0 }
    }

    pub fn get(&self, x: u32) -> u32 {
        let mut a = x.wrapping_mul(4294967291);
        a = a.wrapping_add(self.seed);
        a = a.wrapping_pow(5);
        a ^= a.rotate_right(5);
        a ^= a.rotate_left(9);
        a
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        self.index += 1;
        self.get(self.index)
    }

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

pub fn perlin_2d(noise: &Noise, pos: Vec2) -> f32 {
    fn dot_gradient(noise: &Noise, cell: Vec2, pos: Vec2) -> f32 {
        let gradient = Vec2::new(
            noise.get(2 * cell.x as u32) as f32,
            noise.get(2 * cell.x as u32 + 1) as f32,
        )
        .normalize();

        let distance_vector = pos - cell;

        distance_vector.dot(gradient)
    }

    let grid_cell_start = pos.floor();
    let grid_cell_end = grid_cell_start + Vec2::new(1.0, 1.0);

    let lerp_weights = pos - grid_cell_start;

    let top_left_gradient = dot_gradient(noise, grid_cell_start, pos);
    let top_right_gradient =
        dot_gradient(noise, Vec2::new(grid_cell_start.x, grid_cell_end.y), pos);
    let bottom_left_gradient =
        dot_gradient(noise, Vec2::new(grid_cell_end.x, grid_cell_start.y), pos);
    let bottom_right_gradient = dot_gradient(noise, grid_cell_end, pos);

    let top_lerp = lerp(top_left_gradient, top_right_gradient, lerp_weights.x);
    let bottom_lerp = lerp(bottom_left_gradient, bottom_right_gradient, lerp_weights.x);

    lerp(top_lerp, bottom_lerp, lerp_weights.y)
}
