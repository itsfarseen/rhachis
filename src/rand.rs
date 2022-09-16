use std::{
    ops::{Bound, RangeBounds},
    time::SystemTime,
};

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
        x.wrapping_add(479001599)
            ^ x.wrapping_pow(5)
            ^ self.seed.rotate_left(7)
            ^ !x.rotate_left(20)
            ^ self.seed.rotate_right(15)
            ^ x.rotate_right((self.seed % 3).wrapping_add(1))
            ^ self
                .seed
                .rotate_right(x.checked_rem(self.seed).unwrap_or(1))
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
