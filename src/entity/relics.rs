use serde::{Deserialize, Serialize};

use crate::rng::{Luck, RandomType, Rng};

// A permanent prize that biases one random stream. Luck is additive and clamped by
// the roll itself, so stacking relics can never produce an out-of-range draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relic {
    pub name: String,
    pub target: RandomType,
    pub luck: f64,
}

impl Relic {
    pub fn new(name: String, target: RandomType, luck: f64) -> Self {
        Self { name, target, luck }
    }
}

// Every relic the player has earned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relics {
    items: Vec<Relic>,
}

impl Relics {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn items(&self) -> &[Relic] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn add(&mut self, relic: Relic) {
        self.items.push(relic);
    }

    // Aggregates every relic into a single luck table.
    pub fn luck(&self) -> Luck {
        self.items.iter().fold(Luck::zero(), |luck, relic| {
            luck.biased(relic.target, relic.luck)
        })
    }

    // Copies the aggregate onto the RNG. Call after any change to the relics,
    // including loading a save, so the RNG bias stays in sync.
    pub fn apply(&self, rng: &mut Rng) {
        rng.set_luck(self.luck());
    }
}

impl Default for Relics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Relic, Relics};
    use crate::rng::{RandomType, Rng};

    #[test]
    fn aggregate_luck_only_touches_the_targeted_streams() {
        let mut relics = Relics::new();
        relics.add(Relic::new("Ember".to_string(), RandomType::Loot, 0.1));
        relics.add(Relic::new("Prism".to_string(), RandomType::Loot, 0.05));
        relics.add(Relic::new("Sigil".to_string(), RandomType::Discover, 0.2));

        let luck = relics.luck();
        assert!((luck.get(RandomType::Loot) - 0.15).abs() < f64::EPSILON);
        assert!((luck.get(RandomType::Discover) - 0.2).abs() < f64::EPSILON);
        assert!((luck.get(RandomType::Generic)).abs() < f64::EPSILON);

        let mut rng = Rng::new([9; 16]);
        relics.apply(&mut rng);
        assert!((rng.luck(RandomType::Loot) - 0.15).abs() < f64::EPSILON);
    }
}
