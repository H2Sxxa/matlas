use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    context::NodeContext,
    node::{Direction, Node, Pos},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: HashMap<Pos, Node>,
    size: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    PositionOutOfBounds { pos: Pos, size: (usize, usize) },
    OutputRestoreConflict { pos: Pos, item: crate::item::Item },
}

impl Graph {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            nodes: HashMap::new(),
            size,
        }
    }

    pub fn insert(&mut self, pos: Pos, node: Node) -> Result<Option<Node>, GraphError> {
        if !self.contains_pos(&pos) {
            return Err(GraphError::PositionOutOfBounds {
                pos,
                size: self.size,
            });
        }
        Ok(self.nodes.insert(pos, node))
    }

    pub fn node(&self, pos: &Pos) -> Option<&Node> {
        self.nodes.get(pos)
    }

    pub fn node_mut(&mut self, pos: &Pos) -> Option<&mut Node> {
        self.nodes.get_mut(pos)
    }

    pub fn next_pos(&self, pos: &Pos, direction: Direction) -> Option<Pos> {
        match direction {
            Direction::Right => pos.x.checked_add(1).map(|x| Pos { x, y: pos.y }),
            Direction::Down => pos.y.checked_add(1).map(|y| Pos { x: pos.x, y }),
            Direction::Up => pos.y.checked_sub(1).map(|y| Pos { x: pos.x, y }),
            Direction::Left => pos.x.checked_sub(1).map(|x| Pos { x, y: pos.y }),
        }
        .filter(|next| self.contains_pos(next))
    }

    /// Advances the factory by one simulation step.
    ///
    /// Evaluation happens before movement, and every output is snapshotted before
    /// delivery. An item therefore crosses at most one edge per tick. A full
    /// receiver returns the item to its source, making backpressure lossless.
    pub fn tick(&mut self, context: &mut NodeContext) -> Result<(), GraphError> {
        let positions = self.sorted_positions();

        for pos in &positions {
            if let Some(node) = self.nodes.get_mut(pos) {
                node.eval(context);
            }
        }

        let mut outputs = Vec::new();
        for pos in &positions {
            let Some(direction) = self.nodes.get(pos).map(|node| node.direction) else {
                continue;
            };
            let next = self.next_pos(pos, direction);
            let destination_exists = next.is_some_and(|next| self.nodes.contains_key(&next));
            let Some(node) = self.nodes.get_mut(pos) else {
                continue;
            };
            let Some(item) = node.take_output() else {
                continue;
            };
            if let Some(next) = next.filter(|_| destination_exists) {
                outputs.push(Output {
                    from: *pos,
                    to: next,
                    item,
                    accepted: false,
                });
            } else {
                restore_output(node, *pos, item)?;
            }
        }

        let mut remaining_capacity = self
            .nodes
            .iter()
            .map(|(pos, node)| (*pos, node.input_capacity()))
            .collect::<HashMap<_, _>>();
        for output in &mut outputs {
            let capacity = remaining_capacity
                .get_mut(&output.to)
                .expect("output destinations are snapshotted from graph nodes");
            if *capacity > 0 {
                *capacity -= 1;
                output.accepted = true;
            }
        }

        while reject_conflicting_transport_inputs(&mut outputs, &self.nodes) {}

        for output in outputs.iter().filter(|output| !output.accepted) {
            let source = self
                .nodes
                .get_mut(&output.from)
                .expect("output sources are snapshotted from graph nodes");
            restore_output(source, output.from, output.item.clone())?;
        }

        for output in outputs.into_iter().filter(|output| output.accepted) {
            let rejected = self
                .nodes
                .get_mut(&output.to)
                .expect("output destinations are snapshotted from graph nodes")
                .accept(output.item);
            if let Some(item) = rejected {
                let source = self
                    .nodes
                    .get_mut(&output.from)
                    .expect("output sources are snapshotted from graph nodes");
                restore_output(source, output.from, item)?;
            }
        }

        Ok(())
    }

    fn contains_pos(&self, pos: &Pos) -> bool {
        pos.x < self.size.0 && pos.y < self.size.1
    }

    fn sorted_positions(&self) -> Vec<Pos> {
        let mut positions = self.nodes.keys().copied().collect::<Vec<_>>();
        positions.sort_unstable_by_key(|pos| (pos.y, pos.x));
        positions
    }
}

#[derive(Debug, Clone)]
struct Output {
    from: Pos,
    to: Pos,
    item: crate::item::Item,
    accepted: bool,
}

fn reject_conflicting_transport_inputs(outputs: &mut [Output], nodes: &HashMap<Pos, Node>) -> bool {
    let blocked_sources = outputs
        .iter()
        .filter(|output| !output.accepted)
        .map(|output| output.from)
        .collect::<HashSet<_>>();
    let mut changed = false;

    for output in outputs.iter_mut() {
        if output.accepted
            && blocked_sources.contains(&output.to)
            && nodes
                .get(&output.to)
                .expect("output destinations are snapshotted from graph nodes")
                .is_transport()
        {
            output.accepted = false;
            changed = true;
        }
    }

    changed
}

fn restore_output(node: &mut Node, pos: Pos, item: crate::item::Item) -> Result<(), GraphError> {
    node.restore_output(item)
        .map_err(|item| GraphError::OutputRestoreConflict { pos, item })
}

#[cfg(test)]
mod tests {
    use crate::{
        context::NodeContext,
        entity::{atlas::Atlas, repo::Repo},
        node::{
            Direction, Node, NodeObject, Pos, generator::GeneratorNode, inbound::InBoundNode,
            mixturer::MixturerNode, outbound::OutBoundNode, transport::TransportNode,
        },
        rng::Rng,
    };

    use super::Graph;

    fn context() -> NodeContext {
        NodeContext::new(Atlas::new(), Repo::new(), Rng::new([0; 16]))
    }

    #[test]
    fn moves_items_one_edge_per_tick_and_stores_them() {
        let mut graph = Graph::new((3, 1));
        graph
            .insert(
                Pos { x: 0, y: 0 },
                Node::new(
                    NodeObject::Generator(GeneratorNode::new()),
                    Direction::Right,
                ),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 1, y: 0 },
                Node::new(
                    NodeObject::Transport(TransportNode::new()),
                    Direction::Right,
                ),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 2, y: 0 },
                Node::new(NodeObject::Inbound(InBoundNode::new()), Direction::Right),
            )
            .unwrap();
        let mut context = context();

        graph.tick(&mut context).unwrap();
        assert_eq!(context.repo.count(0), 0);
        graph.tick(&mut context).unwrap();
        assert_eq!(context.repo.count(0), 0);
        graph.tick(&mut context).unwrap();
        assert_eq!(context.repo.count(0), 1);
    }

    #[test]
    fn mixer_fills_both_input_slots_from_one_generator() {
        let mut graph = Graph::new((3, 1));
        graph
            .insert(
                Pos { x: 0, y: 0 },
                Node::new(
                    NodeObject::Generator(GeneratorNode::new()),
                    Direction::Right,
                ),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 1, y: 0 },
                Node::new(NodeObject::Mixer(MixturerNode::new()), Direction::Right),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 2, y: 0 },
                Node::new(NodeObject::Inbound(InBoundNode::new()), Direction::Right),
            )
            .unwrap();
        let mut context = context();

        for _ in 0..4 {
            graph.tick(&mut context).unwrap();
        }

        assert_eq!(context.repo.count(0), 0);
        assert_eq!(context.repo.count(1), 1);
    }

    #[test]
    fn mixer_accepts_inputs_arriving_on_different_ticks() {
        let mut graph = Graph::new((3, 3));
        graph
            .insert(
                Pos { x: 0, y: 0 },
                Node::new(NodeObject::Outbound(OutBoundNode::new(0)), Direction::Right),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 1, y: 0 },
                Node::new(NodeObject::Mixer(MixturerNode::new()), Direction::Right),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 1, y: 2 },
                Node::new(NodeObject::Generator(GeneratorNode::new()), Direction::Up),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 1, y: 1 },
                Node::new(NodeObject::Transport(TransportNode::new()), Direction::Up),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 2, y: 0 },
                Node::new(NodeObject::Inbound(InBoundNode::new()), Direction::Right),
            )
            .unwrap();
        let mut context = context();
        context.repo.inbound(0);

        for _ in 0..4 {
            graph.tick(&mut context).unwrap();
        }

        assert_eq!(context.repo.count(0), 0);
        assert_eq!(context.repo.count(1), 1);
    }

    #[test]
    fn outbound_transfers_a_repository_item() {
        let mut graph = Graph::new((2, 1));
        graph
            .insert(
                Pos { x: 0, y: 0 },
                Node::new(NodeObject::Outbound(OutBoundNode::new(0)), Direction::Right),
            )
            .unwrap();
        graph
            .insert(
                Pos { x: 1, y: 0 },
                Node::new(NodeObject::Inbound(InBoundNode::new()), Direction::Right),
            )
            .unwrap();
        let mut context = context();
        context.repo.inbound(0);

        graph.tick(&mut context).unwrap();
        assert_eq!(context.repo.count(0), 0);
        graph.tick(&mut context).unwrap();
        assert_eq!(context.repo.count(0), 1);
    }
}
