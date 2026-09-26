use std::collections::HashMap;

use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

use crate::item::{CapitalizedItem, Item, ItemId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Atlas {
    names: HashMap<ItemId, String>,
    levels: HashMap<ItemId, usize>,
    values: HashMap<ItemId, usize>,
    receipes: HashMap<(Item, Item), Item>,
    increment: ItemId,
}

impl Atlas {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            levels: HashMap::new(),
            values: HashMap::new(),
            receipes: HashMap::new(),
            increment: 1,
        }
    }

    pub fn init(&mut self, rng: &mut impl Rng) -> Item {
        let name = generate_name(rng);
        let value = 1;
        let level = 1;
        self.names.insert(0, name);
        self.values.insert(0, value);
        self.levels.insert(0, level);

        Item { id: 0 }
    }

    pub fn base(&mut self, rng: &mut impl Rng) -> Item {
        if self.names.contains_key(&0) {
            Item { id: 0 }
        } else {
            self.init(rng)
        }
    }

    pub fn mix(&mut self, rng: &mut impl Rng, input: &(Item, Item)) -> Item {
        let key = canonical_pair(&input.0, &input.1);

        if let Some(item) = self.receipes.get(&key) {
            return item.clone();
        }

        let item = self.discover(rng, input);

        self.receipes.entry(input.clone()).or_insert(item).clone()
    }

    fn discover(&mut self, rng: &mut impl Rng, input: &(Item, Item)) -> Item {
        let (right, left) = input;
        // Level spread, more like a lower level
        let right_level = self.safe_level(right.id);
        let left_level = self.safe_level(left.id);

        let min_level = right_level.min(left_level);
        let max_level = right_level.max(left_level) + 1;
        let level = min_level + rng.random_range(0..=max_level - min_level);

        let item = Item { id: self.increment };
        self.increment += 1;

        self.names.insert(item.id, generate_name(rng));
        self.values.insert(
            item.id,
            generate_value(
                self.safe_value(right.id),
                self.safe_value(left.id),
                level,
                rng,
            ),
        );
        self.levels.insert(item.id, level);
        self.receipes
            .insert((right.clone(), left.clone()), item.clone());

        item
    }

    pub fn safe_name(&self, id: ItemId) -> String {
        self.names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("Unknown Item #{}", id))
    }

    pub fn safe_value(&self, id: ItemId) -> usize {
        self.values.get(&id).copied().unwrap_or(1)
    }

    pub fn safe_level(&self, id: ItemId) -> usize {
        self.levels.get(&id).copied().unwrap_or(1)
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}
fn canonical_pair(a: &Item, b: &Item) -> (Item, Item) {
    if a.id <= b.id {
        (a.clone(), b.clone())
    } else {
        (b.clone(), a.clone())
    }
}
fn generate_value(left: usize, right: usize, level: usize, rng: &mut impl Rng) -> usize {
    let parent = (left + right) as f64;
    let base = parent * 1.5;
    let spread = 0.15 + 0.65 * (1.0 - (-0.3 * level as f64).exp());
    let factor = 1.0 + rng.random_range(-spread..=spread);
    (base * factor).round().max(1.0) as usize
}

const SYLLABLES: &[&str] = &[
    "ka", "ke", "ki", "ko", "va", "ve", "vi", "vo", "ra", "re", "ri", "ro", "za", "ze", "zi", "zo",
    "na", "ne", "ni", "no", "la", "le", "li", "lo", "ma", "me", "mi", "mo", "ta", "te", "ti", "to",
];

const SUFFIXES: &[&str] = &["", "", "", "um", "on", "ite", "ex", "ium", "ar", "is"];

fn generate_name(rng: &mut impl Rng) -> String {
    let mut name = String::new();
    let syllable_count = rng.random_range(1..=3);
    for _ in 0..=syllable_count {
        name.push_str(SYLLABLES[rng.random_range(0..SYLLABLES.len())]);
    }
    name.push_str(SUFFIXES[rng.random_range(0..SUFFIXES.len())]);
    name
}

pub trait ItemExtAtlas {
    fn name_atlas(&self, atlas: &Atlas) -> String;
    fn value_atlas(&self, atlas: &Atlas) -> usize;
    fn level_atlas(&self, atlas: &Atlas) -> usize;
    fn capitalize_atlas(&self, atlas: &Atlas) -> CapitalizedItem;
}

impl ItemExtAtlas for Item {
    fn name_atlas(&self, atlas: &Atlas) -> String {
        atlas.safe_name(self.id)
    }

    fn value_atlas(&self, atlas: &Atlas) -> usize {
        atlas.safe_value(self.id)
    }

    fn level_atlas(&self, atlas: &Atlas) -> usize {
        atlas.safe_level(self.id)
    }

    fn capitalize_atlas(&self, atlas: &Atlas) -> CapitalizedItem {
        CapitalizedItem {
            name: self.name_atlas(atlas),
            item: self.clone(),
            value: self.value_atlas(atlas),
            level: self.level_atlas(atlas),
        }
    }
}

pub trait Mix {
    fn mix(&self, rng: &mut impl Rng, atlas: &mut Atlas) -> Self;
    fn fmix(&self, rng: &mut impl Rng, atlas: &mut Atlas) -> String;
}

impl Mix for (Item, Item) {
    fn mix(&self, rng: &mut impl Rng, atlas: &mut Atlas) -> Self {
        let mixed_item = atlas.mix(rng, self);
        (self.0.clone(), mixed_item)
    }

    fn fmix(&self, rng: &mut impl Rng, atlas: &mut Atlas) -> String {
        let mixed_item = self.mix(rng, atlas);
        format!(
            "Recipe: {}({}) + {}({}) => {}({})",
            self.0.capitalize_atlas(atlas),
            self.0.value_atlas(atlas),
            self.1.capitalize_atlas(atlas),
            self.1.value_atlas(atlas),
            mixed_item.1.capitalize_atlas(atlas),
            mixed_item.1.value_atlas(atlas)
        )
    }
}

#[cfg(test)]
mod atlas_tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_name() {
        let mut rng = rand::rngs::Xoshiro128PlusPlus::from_seed([0; 16]);
        let name = generate_name(&mut rng);
        println!("Generated name: {}", name);
    }

    #[test]
    fn test_mix() {
        let mut rng = rand::rngs::Xoshiro128PlusPlus::from_seed([0; 16]);
        let mut atlas = Atlas::new();
        let base = atlas.init(&mut rng);
        let child = atlas.mix(&mut rng, &(base.clone(), base.clone()));
        println!(
            "{}",
            (base.clone(), base.clone()).fmix(&mut rng, &mut atlas)
        );
        println!(
            "{}",
            (base.clone(), child.clone()).fmix(&mut rng, &mut atlas)
        );
        println!(
            "{}",
            (child.clone(), base.clone()).fmix(&mut rng, &mut atlas)
        );
        println!(
            "{}",
            (child.clone(), child.clone()).fmix(&mut rng, &mut atlas)
        );
    }
}
