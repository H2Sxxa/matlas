// Nodes own their local input and output buffers. Graph owns movement between nodes.
use std::ops::Add;

use crate::{context::NodeContext, item::Item};
use serde::{Deserialize, Serialize};
pub mod generator;
pub mod inbound;
pub mod mixturer;
pub mod outbound;
pub mod belt;

pub trait NodeBehavior {
    const NAME: &'static str;
    fn eval(&mut self, _context: &mut NodeContext) {}
    fn take_output(&mut self) -> Option<Item> {
        None
    }
    fn restore_output(&mut self, item: Item) -> Result<(), Item> {
        Err(item)
    }
    fn accept(&mut self, item: Item) -> Option<Item> {
        Some(item)
    }
    fn input_capacity(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeObject {
    Generator(generator::GeneratorNode),
    Inbound(inbound::InBoundNode),
    Outbound(outbound::OutBoundNode),
    Mixer(mixturer::MixturerNode),
    Transport(belt::BeltNode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub object: NodeObject,
    pub direction: Direction,
}

impl Node {
    pub fn new(object: NodeObject, direction: Direction) -> Self {
        Self { object, direction }
    }

    pub fn eval(&mut self, context: &mut NodeContext) {
        match &mut self.object {
            NodeObject::Generator(node) => node.eval(context),
            NodeObject::Inbound(node) => node.eval(context),
            NodeObject::Outbound(node) => node.eval(context),
            NodeObject::Mixer(node) => node.eval(context),
            NodeObject::Transport(node) => node.eval(context),
        }
    }

    pub fn take_output(&mut self) -> Option<Item> {
        match &mut self.object {
            NodeObject::Generator(node) => node.take_output(),
            NodeObject::Inbound(node) => node.take_output(),
            NodeObject::Outbound(node) => node.take_output(),
            NodeObject::Mixer(node) => node.take_output(),
            NodeObject::Transport(node) => node.take_output(),
        }
    }

    pub fn restore_output(&mut self, item: Item) -> Result<(), Item> {
        match &mut self.object {
            NodeObject::Generator(node) => node.restore_output(item),
            NodeObject::Inbound(node) => node.restore_output(item),
            NodeObject::Outbound(node) => node.restore_output(item),
            NodeObject::Mixer(node) => node.restore_output(item),
            NodeObject::Transport(node) => node.restore_output(item),
        }
    }

    pub fn accept(&mut self, item: Item) -> Option<Item> {
        match &mut self.object {
            NodeObject::Generator(node) => node.accept(item),
            NodeObject::Inbound(node) => node.accept(item),
            NodeObject::Outbound(node) => node.accept(item),
            NodeObject::Mixer(node) => node.accept(item),
            NodeObject::Transport(node) => node.accept(item),
        }
    }

    pub fn input_capacity(&self) -> usize {
        match &self.object {
            NodeObject::Generator(node) => node.input_capacity(),
            NodeObject::Inbound(node) => node.input_capacity(),
            NodeObject::Outbound(node) => node.input_capacity(),
            NodeObject::Mixer(node) => node.input_capacity(),
            NodeObject::Transport(node) => node.input_capacity(),
        }
    }

    pub fn is_transport(&self) -> bool {
        matches!(self.object, NodeObject::Transport(_))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Right,
    Down,
    Up,
    Left,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Add<Direction> for &Pos {
    type Output = Pos;

    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::Right => Pos {
                x: self.x + 1,
                y: self.y,
            },
            Direction::Down => Pos {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Up => Pos {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Pos {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
}

impl Add for Direction {
    type Output = usize;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Opposite direction, count as 0
            (Direction::Right, Direction::Left) => 0,
            (Direction::Left, Direction::Right) => 0,
            (Direction::Down, Direction::Up) => 0,
            (Direction::Up, Direction::Down) => 0,
            // Same direction, count as 2
            (Direction::Right, Direction::Right) => 2,
            (Direction::Down, Direction::Down) => 2,
            (Direction::Up, Direction::Up) => 2,
            (Direction::Left, Direction::Left) => 2,
            _ => 1,
        }
    }
}
