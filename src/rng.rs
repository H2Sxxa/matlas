use std::ops::Range;

use rand::{RngExt, SeedableRng, rngs::Xoshiro128PlusPlus};
use serde::{Deserialize, Serialize};

// Selects one of the independent random streams. Keeping the streams apart stops a
// change in one kind of roll from shifting the outcome of another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RandomType {
    Generic,
    Loot,
    Discover,
    Sell,
}

// Additive bias applied to uniform rolls, one entry per stream.
//
// A positive bias pushes a roll toward the top of its range: higher item levels,
// richer item values and rarer loot. Relics are the only source of luck.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Luck {
    generic: f64,
    loot: f64,
    discover: f64,
    sell: f64,
}

impl Luck {
    pub fn zero() -> Self {
        Self {
            generic: 0.0,
            loot: 0.0,
            discover: 0.0,
            sell: 0.0,
        }
    }

    pub fn get(&self, random_type: RandomType) -> f64 {
        match random_type {
            RandomType::Generic => self.generic,
            RandomType::Loot => self.loot,
            RandomType::Discover => self.discover,
            RandomType::Sell => self.sell,
        }
    }

    // Returns a new table with `amount` added to a single stream.
    pub fn biased(&self, random_type: RandomType, amount: f64) -> Luck {
        match random_type {
            RandomType::Generic => Luck {
                generic: self.generic + amount,
                ..*self
            },
            RandomType::Loot => Luck {
                loot: self.loot + amount,
                ..*self
            },
            RandomType::Discover => Luck {
                discover: self.discover + amount,
                ..*self
            },
            RandomType::Sell => Luck {
                sell: self.sell + amount,
                ..*self
            },
        }
    }
}

// We need to adjust the probabilities based on the relics, so create a new struct to hold the RNG state and methods.
#[derive(Debug, Serialize, Deserialize)]
pub struct Rng {
    seed: [u8; 16],
    pub generic: Xoshiro128PlusPlus,
    pub discover: Xoshiro128PlusPlus,
    pub loot: Xoshiro128PlusPlus,
    pub sell: Xoshiro128PlusPlus,
    luck: Luck,
}

impl Rng {
    pub fn seed(&self) -> [u8; 16] {
        self.seed
    }

    pub fn new(seed: [u8; 16]) -> Self {
        Self {
            seed,
            discover: Xoshiro128PlusPlus::from_seed(derive_seed(seed, b"discover")),
            generic: Xoshiro128PlusPlus::from_seed(derive_seed(seed, b"generic")),
            loot: Xoshiro128PlusPlus::from_seed(derive_seed(seed, b"loot")),
            sell: Xoshiro128PlusPlus::from_seed(derive_seed(seed, b"sell")),
            luck: Luck::zero(),
        }
    }

    pub fn luck(&self, random_type: RandomType) -> f64 {
        self.luck.get(random_type)
    }

    // Replaces the cached bias table, normally with the aggregate of every relic.
    pub fn set_luck(&mut self, luck: Luck) {
        self.luck = luck;
    }

    // Draws a uniform value in [0.0, 1.0] with the stream bias applied.
    pub fn roll(&mut self, random_type: RandomType) -> f64 {
        (self.next_unit(random_type) + self.luck.get(random_type)).clamp(0.0, 1.0)
    }

    pub fn roll_range(&mut self, random_type: RandomType, range: Range<f64>) -> f64 {
        range.start + self.roll(random_type) * (range.end - range.start)
    }

    pub fn roll_index(&mut self, random_type: RandomType, len: usize) -> usize {
        index_of(self.roll(random_type), len)
    }

    // Draws without the stream bias. Used where luck would distort a choice rather
    // than improve an outcome, such as picking the next goal.
    pub fn roll_index_pure(&mut self, random_type: RandomType, len: usize) -> usize {
        index_of(self.next_unit(random_type), len)
    }

    fn next_unit(&mut self, random_type: RandomType) -> f64 {
        match random_type {
            RandomType::Generic => self.generic.random::<f64>(),
            RandomType::Loot => self.loot.random::<f64>(),
            RandomType::Discover => self.discover.random::<f64>(),
            RandomType::Sell => self.sell.random::<f64>(),
        }
    }
}

fn index_of(unit: f64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    ((unit * len as f64) as usize).min(len - 1)
}

fn derive_seed(master: [u8; 16], namespace: &[u8]) -> [u8; 16] {
    let mut seed = [0u8; 16];
    let mut hasher = blake3::Hasher::new();
    hasher.update(&master);
    hasher.update(namespace);
    let hash = hasher.finalize();
    seed.copy_from_slice(&hash.as_bytes()[..16]);
    seed
}
