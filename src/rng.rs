use std::ops::Range;

use rand::{RngExt, SeedableRng, rngs::SmallRng};

// We need to adjust the probabilities based on the relics, so create a new struct to hold the RNG state and methods.
#[derive(Debug)]
pub struct Rng {
    seed: [u8; 32],
    pub generic: SmallRng,
    pub discover: SmallRng,
    pub loot: SmallRng,
}

impl Rng {
    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            seed,
            discover: SmallRng::from_seed(derive_seed(seed, b"discover")),
            generic: SmallRng::from_seed(derive_seed(seed, b"generic")),
            loot: SmallRng::from_seed(derive_seed(seed, b"loot")),
        }
    }
    pub fn random_generic(&mut self) -> f64 {
        self.generic.random()
    }

    pub fn random_range_generic(&mut self, range: Range<f64>) -> f64 {
        self.generic.random_range(range)
    }

    pub fn random_range(&mut self, random_type: RandomType, range: Range<f64>) -> f64 {
        match random_type {
            RandomType::Generic => self.generic.random_range(range),
            RandomType::Loot => self.loot.random_range(range),
            RandomType::Discover => self.discover.random_range(range),
        }
    }

    pub fn random(&mut self, random_type: RandomType) -> f64 {
        match random_type {
            RandomType::Generic => self.generic.random(),
            RandomType::Loot => self.loot.random(),
            RandomType::Discover => self.discover.random(),
        }
    }
}

pub enum RandomType {
    Generic,
    Loot,
    Discover,
}

fn derive_seed(master: [u8; 32], namespace: &[u8]) -> [u8; 32] {
    let mut seed = [0u8; 32];
    let mut hasher = blake3::Hasher::new();
    hasher.update(&master);
    hasher.update(namespace);
    let hash = hasher.finalize();
    seed.copy_from_slice(hash.as_bytes());
    seed
}
