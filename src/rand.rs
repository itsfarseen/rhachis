use std::time::SystemTime;

pub struct Noise {
    seed: u32,
    index: u32,
}

impl Noise {
    pub fn new() -> Noise {
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        Self {
            seed,
            index: 0,
        }
    }

    pub fn from_seed(seed: u32) -> Noise {
        Self {
            seed,
            index: 0,
        }
    }

    pub fn get(&self, x: u32) -> u32 {
        x.wrapping_add(479001599)
            ^ x.wrapping_pow(5)
            ^ self.seed.rotate_left(7)
            ^ !x.rotate_left(20)
            ^ self.seed.rotate_right(15)
            ^ x.rotate_right((self.seed % 3).wrapping_add(1))
            ^ self.seed.rotate_right(x.checked_rem(self.seed).unwrap_or(1))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        self.index += 1;
        self.get(self.index)
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self::new()
    }
}
