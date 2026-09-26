pub mod relic;

use serde::{Deserialize, Serialize};

use crate::{
    entity::{
        machines::{MachineKind, Machines},
        relics::Relic,
    },
    rng::{RandomType, Rng},
};

// Reward tiers. The odds are drawn from the loot stream, so relics that target
// loot shift the spread toward the rarer tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
}

// How many rewards are offered after a goal is completed.
pub const OFFER_COUNT: usize = 3;

impl Rarity {
    // Luck added to a relic of this tier.
    pub fn luck(&self) -> f64 {
        match self {
            Rarity::Common => 0.05,
            Rarity::Uncommon => 0.10,
            Rarity::Rare => 0.20,
        }
    }

    // How many cells a graph reward of this tier adds on one axis.
    pub fn graph_span(&self) -> usize {
        match self {
            Rarity::Common => 1,
            Rarity::Uncommon => 2,
            Rarity::Rare => 3,
        }
    }
}

// Extra cells added to the factory grid. Only one axis grows at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphExpansion {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reward {
    Relic(Relic),
    Machine(MachineKind),
    Graph(GraphExpansion),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardOffer {
    pub rarity: Rarity,
    pub reward: Reward,
}

// Draws the rewards a player may pick from after finishing a goal.
//
// Machine rewards are only in the pool while a slot is free, so loot can never
// hand out more machines than the factory can hold.
pub fn roll_offers(rng: &mut Rng, machines: &Machines, count: usize) -> Vec<RewardOffer> {
    (0..count).map(|_| roll_offer(rng, machines)).collect()
}

fn roll_offer(rng: &mut Rng, machines: &Machines) -> RewardOffer {
    let rarity = roll_rarity(rng);
    let reward = roll_reward(rng, machines, rarity);
    RewardOffer { rarity, reward }
}

fn roll_rarity(rng: &mut Rng) -> Rarity {
    let roll = rng.roll(RandomType::Loot);
    if roll < 0.60 {
        Rarity::Common
    } else if roll < 0.90 {
        Rarity::Uncommon
    } else {
        Rarity::Rare
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RewardKind {
    Relic,
    Machine,
    Graph,
}

fn roll_reward(rng: &mut Rng, machines: &Machines, rarity: Rarity) -> Reward {
    let kinds = available_kinds(machines);
    let kind = kinds[rng.roll_index(RandomType::Loot, kinds.len())];
    match kind {
        RewardKind::Relic => Reward::Relic(relic::roll(rng, rarity)),
        RewardKind::Machine => Reward::Machine(
            MachineKind::ALL[rng.roll_index(RandomType::Loot, MachineKind::ALL.len())],
        ),
        RewardKind::Graph => Reward::Graph(roll_expansion(rng, rarity)),
    }
}

fn available_kinds(machines: &Machines) -> Vec<RewardKind> {
    let mut kinds = vec![RewardKind::Relic, RewardKind::Graph];
    if machines.can_grant() {
        kinds.push(RewardKind::Machine);
    }
    kinds
}

fn roll_expansion(rng: &mut Rng, rarity: Rarity) -> GraphExpansion {
    let span = rarity.graph_span();
    if rng.roll_index(RandomType::Loot, 2) == 0 {
        GraphExpansion {
            width: span,
            height: 0,
        }
    } else {
        GraphExpansion {
            width: 0,
            height: span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OFFER_COUNT, Reward, roll_offers};
    use crate::{entity::machines::Machines, rng::Rng};

    #[test]
    fn offers_a_machine_only_while_a_slot_is_free() {
        let mut rng = Rng::new([3; 16]);

        let full = Machines::new(0);
        for _ in 0..32 {
            let offers = roll_offers(&mut rng, &full, OFFER_COUNT);
            assert!(
                offers
                    .iter()
                    .all(|offer| !matches!(offer.reward, Reward::Machine(_))),
                "a full factory must never be offered a machine"
            );
        }

        let open = Machines::new(4);
        let offered = (0..64).any(|_| {
            roll_offers(&mut rng, &open, OFFER_COUNT)
                .iter()
                .any(|offer| matches!(offer.reward, Reward::Machine(_)))
        });
        assert!(offered, "an open slot should eventually offer a machine");
    }
}
