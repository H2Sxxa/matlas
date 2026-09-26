// Definition of Node
// 1. Each Node has a direction, and can be connected to other nodes
// 2. Each Node has a type, which defines its behavior
// 3. Each Node will try to push its output to its direction next node, if it exists
// 4. Each Node will eval to generate its output when it got push and satify its input requirements, then start countdown,when reamning 0, it will push.
use std::ops::Add;

use crate::context::NodeContext;
use serde::{Deserialize, Serialize};
pub mod generator;
pub mod inbound;
pub mod mixturer;
pub mod outbound;
pub mod transport;

pub trait NodeBehavior {
    const NAME: &'static str;
    fn eval(&mut self, _context: &mut NodeContext) {}
    fn push(&mut self, _context: &mut NodeContext, _next: Option<NodeObject>) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeObject {
    Generator(generator::GeneratorNode),
    Inbound(inbound::InBoundNode),
    Outbound(outbound::OutBoundNode),
    Mixer(mixturer::MixturerNode),
    Transport(transport::TransportNode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub object: NodeObject,
    pub direction: Direction,
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
