use crate::{
    entity::relics::Relic,
    loot::Rarity,
    rng::{RandomType, Rng},
};

// Relics only target streams where a high roll is a real benefit. Generic is left
// out: it drives goal draws and names, where luck would only skew choices.
const TARGETS: [RandomType; 3] = [RandomType::Loot, RandomType::Discover, RandomType::Sell];

const ADJECTIVES: &[&str] = &[
    "Faint", "Hollow", "Bright", "Wound", "Still", "Bitter", "Pale", "Deep",
];

const NOUNS: &[&str] = &[
    "Sigil", "Prism", "Ember", "Loop", "Ward", "Spindle", "Anchor", "Bloom",
];

// Rolls a relic of the given rarity. Rarity sets the strength of the bias; the
// stream it targets and its name are random.
pub fn roll(rng: &mut Rng, rarity: Rarity) -> Relic {
    let target = TARGETS[rng.roll_index(RandomType::Loot, TARGETS.len())];
    let name = generate_name(rng);
    Relic::new(name, target, rarity.luck())
}

fn generate_name(rng: &mut Rng) -> String {
    let adjective = ADJECTIVES[rng.roll_index(RandomType::Generic, ADJECTIVES.len())];
    let noun = NOUNS[rng.roll_index(RandomType::Generic, NOUNS.len())];
    format!("{adjective} {noun}")
}
