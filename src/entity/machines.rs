use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::node::{
    Direction, Node, NodeObject, belt::BeltNode, distributor::DistributorNode,
    generator::GeneratorNode, inbound::InBoundNode, mixturer::MixturerNode, overflow::OverflowNode,
    sell::SellNode,
};

// A machine that can be handed out as loot and placed on the graph.
//
// Outbound terminals are absent on purpose: they are configured with the item they
// ship, so they are part of a layout rather than a prize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MachineKind {
    Generator,
    Belt,
    Mixer,
    Inbound,
    Distributor,
    Overflow,
    Sell,
}

impl MachineKind {
    pub const ALL: [MachineKind; 7] = [
        MachineKind::Generator,
        MachineKind::Belt,
        MachineKind::Mixer,
        MachineKind::Inbound,
        MachineKind::Distributor,
        MachineKind::Overflow,
        MachineKind::Sell,
    ];

    pub fn build(&self, direction: Direction) -> Node {
        let object = match self {
            MachineKind::Generator => NodeObject::Generator(GeneratorNode::new()),
            MachineKind::Belt => NodeObject::Transport(BeltNode::new()),
            MachineKind::Mixer => NodeObject::Mixer(MixturerNode::new()),
            MachineKind::Inbound => NodeObject::Inbound(InBoundNode::new()),
            MachineKind::Distributor => NodeObject::Distributor(DistributorNode::new()),
            MachineKind::Overflow => NodeObject::Overflow(OverflowNode::new()),
            MachineKind::Sell => NodeObject::Sell(SellNode::new()),
        };
        Node::new(object, direction)
    }

    // Maps a placed node back to the machine it was built from. Outbound is a layout
    // terminal rather than a machine reward, so it maps to None.
    pub fn from_object(object: &NodeObject) -> Option<MachineKind> {
        match object {
            NodeObject::Generator(_) => Some(MachineKind::Generator),
            NodeObject::Transport(_) => Some(MachineKind::Belt),
            NodeObject::Mixer(_) => Some(MachineKind::Mixer),
            NodeObject::Inbound(_) => Some(MachineKind::Inbound),
            NodeObject::Distributor(_) => Some(MachineKind::Distributor),
            NodeObject::Overflow(_) => Some(MachineKind::Overflow),
            NodeObject::Sell(_) => Some(MachineKind::Sell),
            NodeObject::Outbound(_) => None,
        }
    }
}

impl Display for MachineKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MachineKind::Generator => "Generator",
            MachineKind::Belt => "Belt",
            MachineKind::Mixer => "Mixer",
            MachineKind::Inbound => "Inbound",
            MachineKind::Distributor => "Distributor",
            MachineKind::Overflow => "Overflow",
            MachineKind::Sell => "Sell",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineError {
    NoFreeSlot { capacity: usize, owned: usize },
    NotInStock { kind: MachineKind, stock: usize },
}

impl Display for MachineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineError::NoFreeSlot { capacity, owned } => write!(
                f,
                "no free machine slot: {owned} of {capacity} slots are taken"
            ),
            MachineError::NotInStock { kind, stock } => {
                write!(f, "no {kind} in stock: {stock} available")
            }
        }
    }
}

impl Error for MachineError {}

// Every machine the player owns, split between the ones waiting in stock and the
// ones already placed on the board. `capacity` is the hard limit that keeps loot
// from handing out machines forever.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machines {
    capacity: usize,
    placed: usize,
    stock: HashMap<MachineKind, usize>,
}

impl Machines {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            placed: 0,
            stock: HashMap::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn placed(&self) -> usize {
        self.placed
    }

    pub fn stock(&self, kind: MachineKind) -> usize {
        self.stock.get(&kind).copied().unwrap_or(0)
    }

    pub fn owned(&self) -> usize {
        self.placed + self.stock.values().sum::<usize>()
    }

    pub fn free_slots(&self) -> usize {
        self.capacity.saturating_sub(self.owned())
    }

    pub fn can_grant(&self) -> bool {
        self.free_slots() > 0
    }

    // Adds a machine to the stock, refusing once every slot is taken.
    pub fn grant(&mut self, kind: MachineKind) -> Result<(), MachineError> {
        if !self.can_grant() {
            return Err(MachineError::NoFreeSlot {
                capacity: self.capacity,
                owned: self.owned(),
            });
        }
        *self.stock.entry(kind).or_insert(0) += 1;
        Ok(())
    }

    // Moves one machine from stock onto the board.
    pub fn install(&mut self, kind: MachineKind) -> Result<(), MachineError> {
        if self.stock(kind) == 0 {
            return Err(MachineError::NotInStock { kind, stock: 0 });
        }
        *self.stock.entry(kind).or_insert(0) -= 1;
        self.placed += 1;
        Ok(())
    }

    // Returns a machine from the board to stock.
    pub fn uninstall(&mut self, kind: MachineKind) {
        self.placed = self.placed.saturating_sub(1);
        *self.stock.entry(kind).or_insert(0) += 1;
    }

    // Raises the slot count, for a reward or relic that widens the factory floor.
    pub fn expand(&mut self, extra_slots: usize) {
        self.capacity += extra_slots;
    }
}

impl Default for Machines {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{MachineError, MachineKind, Machines};

    #[test]
    fn stops_granting_once_every_slot_is_taken() {
        let mut machines = Machines::new(2);

        assert!(machines.can_grant());
        machines
            .grant(MachineKind::Belt)
            .expect("first slot is free");
        machines
            .grant(MachineKind::Mixer)
            .expect("second slot is free");

        assert_eq!(machines.free_slots(), 0);
        assert_eq!(
            machines.grant(MachineKind::Generator),
            Err(MachineError::NoFreeSlot {
                capacity: 2,
                owned: 2
            })
        );
    }

    #[test]
    fn installing_keeps_the_slot_occupied() {
        let mut machines = Machines::new(1);
        machines.grant(MachineKind::Belt).expect("slot is free");

        machines
            .install(MachineKind::Belt)
            .expect("belt is in stock");

        assert_eq!(machines.placed(), 1);
        assert_eq!(machines.free_slots(), 0);
        assert_eq!(
            machines.install(MachineKind::Belt),
            Err(MachineError::NotInStock {
                kind: MachineKind::Belt,
                stock: 0
            })
        );

        machines.uninstall(MachineKind::Belt);
        assert_eq!(machines.stock(MachineKind::Belt), 1);
        assert_eq!(machines.free_slots(), 0);
    }
}
