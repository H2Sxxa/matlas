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
    PositionOutOfBounds {
        pos: Pos,
        size: (usize, usize),
    },
    OutputRestoreConflict {
        pos: Pos,
        item: crate::item::Item,
    },
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
            let Some(directions) = self.nodes.get(pos).map(Node::output_directions) else {
                continue;
            };
            let routes = directions
                .into_iter()
                .filter_map(|direction| {
                    self.next_pos(pos, direction)
                        .filter(|next| self.nodes.contains_key(next))
                        .map(|next| Route {
                            direction,
                            to: next,
                        })
                })
                .collect::<Vec<_>>();
            let Some(node) = self.nodes.get_mut(pos) else {
                continue;
            };
            let Some(item) = node.take_output() else {
                continue;
            };
            if !routes.is_empty() {
                outputs.push(Output {
                    from: *pos,
                    routes,
                    item,
                    route: None,
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
            if let Some(route) = output.routes.iter().find(|route| {
                remaining_capacity
                    .get(&route.to)
                    .is_some_and(|capacity| *capacity > 0)
            }) {
                if let Some(capacity) = remaining_capacity.get_mut(&route.to) {
                    *capacity -= 1;
                    output.route = Some(*route);
                }
            }
        }

        while reject_conflicting_transport_inputs(&mut outputs, &self.nodes) {}

        for output in outputs.iter().filter(|output| output.route.is_none()) {
            if let Some(source) = self.nodes.get_mut(&output.from) {
                restore_output(source, output.from, output.item.clone())?;
            }
        }

        for output in outputs.into_iter().filter(|output| output.route.is_some()) {
            let Some(route) = output.route else { continue };
            let rejected = self
                .nodes
                .get_mut(&route.to)
                .and_then(|node| node.accept(output.item));
            if let Some(item) = rejected {
                if let Some(source) = self.nodes.get_mut(&output.from) {
                    restore_output(source, output.from, item)?;
                }
            } else if let Some(source) = self.nodes.get_mut(&output.from) {
                source.confirm_output(route.direction);
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
    routes: Vec<Route>,
    item: crate::item::Item,
    route: Option<Route>,
}

#[derive(Debug, Clone, Copy)]
struct Route {
    direction: Direction,
    to: Pos,
}

fn reject_conflicting_transport_inputs(outputs: &mut [Output], nodes: &HashMap<Pos, Node>) -> bool {
    let blocked_sources = outputs
        .iter()
        .filter(|output| output.route.is_none())
        .map(|output| output.from)
        .collect::<HashSet<_>>();
    let mut changed = false;

    for output in outputs.iter_mut() {
        if output.route.is_some_and(|route| {
            blocked_sources.contains(&route.to)
                && nodes.get(&route.to).is_some_and(Node::is_transport)
        }) {
            output.route = None;
            changed = true;
        }
    }

    changed
}

fn restore_output(
    node: &mut Node,
    pos: Pos,
    item: crate::item::Item,
) -> Result<(), GraphError> {
    node.restore_output(item)
        .map_err(|item| GraphError::OutputRestoreConflict { pos, item })
}

#[cfg(test)]
mod tests {
    use crate::{
        node::{Direction, Pos},
        test_support::{belt, context, generator, graph, inbound, peek_output, place, ticks},
    };

    use super::GraphError;

    // (0,0) G>  (1,0) B>  (2,0) B>  (3,0) I
    // Legend: G generator | B belt | I inbound | > facing right.
    // Evaluation runs before movement, so an item advances a single edge per tick.
    #[test]
    fn moves_an_item_one_edge_per_tick() {
        let mut graph = graph((4, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, belt(Direction::Right));
        place(&mut graph, Pos { x: 2, y: 0 }, belt(Direction::Right));
        place(&mut graph, Pos { x: 3, y: 0 }, inbound(Direction::Right));
        let mut context = context();

        for (tick, expected) in [0, 0, 0, 1].into_iter().enumerate() {
            ticks(&mut graph, &mut context, 1);
            assert_eq!(context.repo.count(0), expected, "after tick {}", tick + 1);
        }
    }

    // (1,0) G>
    // (2,1) G^   (2,0) I
    // Legend: G generator | I inbound | > ^ facing.
    // An inbound has no slot limit, so both sources deliver in the same tick.
    #[test]
    fn accepts_one_item_per_source_into_the_same_node() {
        let mut graph = graph((3, 2));
        place(&mut graph, Pos { x: 1, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 2, y: 0 }, inbound(Direction::Right));
        place(&mut graph, Pos { x: 2, y: 1 }, generator(Direction::Up));
        let mut context = context();

        ticks(&mut graph, &mut context, 2);
        assert_eq!(context.repo.count(0), 2);

        ticks(&mut graph, &mut context, 1);
        assert_eq!(context.repo.count(0), 4);
    }

    // (0,0) G>  (1,0) B>  (2,0) .
    // Legend: G generator | B belt | . empty cell | > facing right.
    // The belt cannot drain, so it keeps its item and the generator keeps the item
    // it could not hand over: nothing is dropped or overwritten.
    #[test]
    fn keeps_items_out_of_a_jammed_transport_without_losing_them() {
        let mut graph = graph((3, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, belt(Direction::Right));
        let mut context = context();

        ticks(&mut graph, &mut context, 4);

        assert_eq!(context.repo.count(0), 0);
        assert!(peek_output(&mut graph, Pos { x: 0, y: 0 }).is_some());
        assert!(peek_output(&mut graph, Pos { x: 1, y: 0 }).is_some());
    }

    // (2,0) lies outside a 2x1 graph.
    // Legend: G generator | > facing right.
    #[test]
    fn rejects_positions_outside_the_graph() {
        let mut graph = graph((2, 1));
        let pos = Pos { x: 2, y: 0 };

        let error = graph.insert(pos, generator(Direction::Right)).unwrap_err();

        assert_eq!(
            error,
            GraphError::PositionOutOfBounds {
                pos,
                size: (2, 1)
            }
        );
    }
}
