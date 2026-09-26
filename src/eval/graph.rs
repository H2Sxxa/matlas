use scc::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::{
    context::NodeContext,
    node::{Direction, Node, NodeObject, Pos},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: HashMap<Pos, Node>,
    size: (usize, usize),
}

#[derive(Debug)]
struct CrossState {
    /// How many branches should arrive at this node.
    branch_num: usize,

    /// How many branches have already arrived.
    executed_branch_num: usize,
}

impl Graph {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            nodes: HashMap::new(),
            size,
        }
    }

    pub fn next_pos(&self, pos: &Pos, direction: Direction) -> Option<Pos> {
        match direction {
            Direction::Right => {
                if pos.x + 1 >= self.size.0 {
                    return None;
                }
            }
            Direction::Down => {
                if pos.y + 1 >= self.size.1 {
                    return None;
                }
            }
            Direction::Up => {
                if pos.y == 0 {
                    return None;
                }
            }
            Direction::Left => {
                if pos.x == 0 {
                    return None;
                }
            }
        }

        Some(pos + direction)
    }

    /// Number of nodes pointing to `pos`.
    pub fn incoming_num(&self, pos: &Pos) -> usize {
        let mut count = 0;

        for direction in [
            Direction::Right,
            Direction::Down,
            Direction::Up,
            Direction::Left,
        ] {
            if let Some(next_pos) = self.next_pos(pos, direction)
                && let Some(node) = self.nodes.get_sync(&next_pos)
                && node.direction + direction == 0
            {
                count += 1;
            }
        }

        count
    }

    /// Generator / Outbound are execution roots.
    pub fn is_root(&self, _pos: &Pos, node: &Node) -> bool {
        matches!(
            node.object,
            NodeObject::Generator(_) | NodeObject::Outbound(_)
        )
    }

    /// Check whether `from` actually connects to `to`.
    fn connected(&self, from: &Pos, to: &Pos) -> bool {
        let Some(node) = self.nodes.get_sync(from) else {
            return false;
        };

        let Some(expected) = self.next_pos(from, node.direction) else {
            return false;
        };

        expected == *to
    }

    /// Execute one node.
    ///
    /// This is deliberately kept separate from graph traversal.
    fn execute_node(&self, pos: &Pos, context: &mut NodeContext) {
        let Some(mut node) = self.nodes.get_sync(pos) else {
            return;
        };

        match &node.get_mut().object {
            NodeObject::Generator(generator_node) => todo!(),
            NodeObject::Inbound(in_bound_node) => todo!(),
            NodeObject::Outbound(out_bound_node) => todo!(),
            NodeObject::Mixer(mixturer_node) => todo!(),
            NodeObject::Transport(transport_node) => todo!(),
        }

    }

    pub fn tick(&self, context: &mut NodeContext) {
        // ------------------------------------------------------------
        // 1. Find cross nodes and roots.
        // ------------------------------------------------------------

        let mut cross_nodes = std::collections::HashMap::<Pos, CrossState>::new();

        let mut roots = Vec::<Pos>::new();

        self.nodes.iter_sync(|pos, node| {
            let branches = self.incoming_num(pos);

            if branches > 1 {
                cross_nodes.insert(
                    *pos,
                    CrossState {
                        branch_num: branches,
                        executed_branch_num: 0,
                    },
                );
            }

            if self.is_root(pos, node) {
                roots.push(*pos);
            }

            true
        });

        // ------------------------------------------------------------
        // 2. Start one execution path from every root.
        // ------------------------------------------------------------

        //
        // A path is:
        //
        //     root -> node -> node -> cross
        //
        // It stops when it reaches a cross node.
        //
        let mut paths = VecDeque::<Pos>::new();

        for root in roots {
            paths.push_back(root);
        }

        // ------------------------------------------------------------
        // 3. Execute paths.
        // ------------------------------------------------------------

        while let Some(start) = paths.pop_front() {
            let mut current = start;

            loop {
                // Execute current node.
                self.execute_node(&current, context);

                // Find next node according to current node's direction.
                let Some(node) = self.nodes.get_sync(&current) else {
                    break;
                };

                let Some(next) = self.next_pos(&current, node.direction) else {
                    break;
                };

                // There is no connected node.
                if !self.nodes.contains_sync(&next) {
                    break;
                }

                // ----------------------------------------------------
                // 4. We reached a cross node.
                // ----------------------------------------------------

                if let Some(cross) = cross_nodes.get_mut(&next) {
                    cross.executed_branch_num += 1;

                    // This branch stops here.
                    //
                    // The cross node will only execute after every
                    // incoming branch has arrived.
                    if cross.executed_branch_num < cross.branch_num {
                        break;
                    }

                    // All branches have arrived.
                    //
                    // Continue execution from the cross node.
                    current = next;
                    continue;
                }

                // ----------------------------------------------------
                // 5. Normal node.
                // ----------------------------------------------------

                current = next;
            }
        }
    }
}
