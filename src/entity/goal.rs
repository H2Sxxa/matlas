use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    entity::{atlas::Atlas, stats::Stats},
    rng::{RandomType, Rng},
};

// A single objective. The target is fixed when the goal is drawn, while progress is
// read live from the atlas and the lifetime stats.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GoalKind {
    // Craft an item of at least this level.
    CraftLevel { level: usize },
    // Craft an item worth at least this much.
    CraftValue { value: usize },
    // Discover at least this many distinct items.
    DiscoverCount { count: usize },
    // Ship items worth at least this much in total.
    Revenue { value: usize },
    // Craft at least this many items in total.
    CraftCount { count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    pub current: usize,
    pub target: usize,
}

impl Display for Progress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.current.min(self.target), self.target)
    }
}

impl GoalKind {
    pub fn progress(&self, atlas: &Atlas, stats: &Stats) -> Progress {
        match *self {
            GoalKind::CraftLevel { level } => Progress {
                current: atlas.max_level(),
                target: level,
            },
            GoalKind::CraftValue { value } => Progress {
                current: atlas.max_value(),
                target: value,
            },
            GoalKind::DiscoverCount { count } => Progress {
                current: atlas.item_count(),
                target: count,
            },
            GoalKind::Revenue { value } => Progress {
                current: stats.revenue(),
                target: value,
            },
            GoalKind::CraftCount { count } => Progress {
                current: stats.crafted(),
                target: count,
            },
        }
    }

    pub fn is_complete(&self, atlas: &Atlas, stats: &Stats) -> bool {
        let progress = self.progress(atlas, stats);
        progress.current >= progress.target
    }
}

impl Display for GoalKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            GoalKind::CraftLevel { level } => write!(f, "Craft a Lv.{level} item"),
            GoalKind::CraftValue { value } => write!(f, "Craft an item worth {value}"),
            GoalKind::DiscoverCount { count } => write!(f, "Discover {count} items"),
            GoalKind::Revenue { value } => write!(f, "Ship items worth {value}"),
            GoalKind::CraftCount { count } => write!(f, "Craft {count} items"),
        }
    }
}

const GOAL_KINDS: usize = 5;

// The chain of goals the player is walking through. Each completed goal makes the
// next one harder, but every target is anchored to the live atlas and stats so a
// goal is always a step beyond what the factory can already do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goals {
    current: GoalKind,
    completed: usize,
}

impl Goals {
    // Draws the first goal. The atlas should already hold its base item.
    pub fn start(atlas: &Atlas, stats: &Stats, rng: &mut Rng) -> Self {
        Self {
            current: draw_goal(0, atlas, stats, rng),
            completed: 0,
        }
    }

    pub fn current(&self) -> GoalKind {
        self.current
    }

    pub fn completed(&self) -> usize {
        self.completed
    }

    pub fn progress(&self, atlas: &Atlas, stats: &Stats) -> Progress {
        self.current.progress(atlas, stats)
    }

    pub fn is_complete(&self, atlas: &Atlas, stats: &Stats) -> bool {
        self.current.is_complete(atlas, stats)
    }

    // Replaces a finished goal with the next, harder one.
    pub fn advance(&mut self, atlas: &Atlas, stats: &Stats, rng: &mut Rng) {
        self.completed += 1;
        self.current = draw_goal(self.completed, atlas, stats, rng);
    }
}

// Draws a goal. `index` is the number of goals already finished, so `tier` grows
// linearly along the chain while the choice of kind and the jitter stay random.
fn draw_goal(index: usize, atlas: &Atlas, stats: &Stats, rng: &mut Rng) -> GoalKind {
    let tier = index + 1;
    match rng.roll_index_pure(RandomType::Generic, GOAL_KINDS) {
        0 => GoalKind::CraftLevel {
            level: atlas.max_level() + 1 + rng.roll_index_pure(RandomType::Generic, tier),
        },
        1 => GoalKind::CraftValue {
            value: value_target(atlas, tier),
        },
        2 => GoalKind::DiscoverCount {
            count: atlas.item_count() + 1 + rng.roll_index_pure(RandomType::Generic, tier),
        },
        3 => GoalKind::Revenue {
            value: revenue_target(atlas, stats, tier, rng),
        },
        _ => GoalKind::CraftCount {
            count: stats.crafted() + 1 + tier + rng.roll_index_pure(RandomType::Generic, tier),
        },
    }
}

// The value floor grows by half a tier per completed goal, rounded up so the first
// goal already asks for more than the item the base recipe produces.
fn value_target(atlas: &Atlas, tier: usize) -> usize {
    let base = atlas.max_value().max(1) as f64;
    (base * (1.0 + 0.5 * tier as f64)).ceil() as usize
}

// Revenue is cumulative, so the target is the current total plus one atlas-scaled
// batch that grows linearly with the tier.
fn revenue_target(atlas: &Atlas, stats: &Stats, tier: usize, rng: &mut Rng) -> usize {
    let unit = atlas.max_value().max(1);
    stats.revenue() + unit * (tier + rng.roll_index_pure(RandomType::Generic, tier))
}

#[cfg(test)]
mod tests {
    use super::Goals;
    use crate::{
        entity::{atlas::Atlas, stats::Stats},
        rng::Rng,
    };

    #[test]
    fn a_working_factory_completes_its_goal_and_draws_a_harder_one() {
        let mut rng = Rng::new([5; 16]);
        let mut atlas = Atlas::new();
        atlas.init(&mut rng.generic);
        let mut stats = Stats::new();
        let mut goals = Goals::start(&atlas, &stats, &mut rng);

        let first = goals.current();
        let first_target = goals.progress(&atlas, &stats).target;

        for _ in 0..1_000 {
            if goals.is_complete(&atlas, &stats) {
                break;
            }
            let left = atlas.base(&mut rng.generic);
            let right = atlas.base(&mut rng.generic);
            atlas.mix(&mut rng.discover, &(left, right), 0.0);
            stats.record_craft();
            stats.record_revenue(10_000.0);
        }

        assert!(
            goals.is_complete(&atlas, &stats),
            "goal {first} should be reachable"
        );

        goals.advance(&atlas, &stats, &mut rng);
        assert_eq!(goals.completed(), 1);
        let next_target = goals.progress(&atlas, &stats).target;
        assert!(
            next_target >= first_target,
            "the next goal should not ask for less: {first_target} -> {next_target}"
        );
    }
}
