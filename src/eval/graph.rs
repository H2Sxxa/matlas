use std::collections::VecDeque;

use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::{
    context::NodeContext,
    node::{Direction, MAX_OUTPUT_DIRECTIONS, Node, Pos},
};

type NodeId = usize;
const EMPTY_NODE: NodeId = usize::MAX;
const MAX_INCOMING: usize = 4;

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: Vec<Node>,
    positions: Vec<Pos>,
    grid: Vec<NodeId>,
    size: (usize, usize),
    order: Vec<NodeId>,
    order_dirty: bool,
    #[serde(skip, default)]
    scratch: TickScratch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    PositionOutOfBounds { pos: Pos, size: (usize, usize) },
    OutputRestoreConflict { pos: Pos, item: crate::item::Item },
}

impl Graph {
    pub fn new(size: (usize, usize)) -> Self {
        let cell_count = size
            .0
            .checked_mul(size.1)
            .expect("graph dimensions overflow usize");

        Self {
            nodes: Vec::new(),
            positions: Vec::new(),
            grid: vec![EMPTY_NODE; cell_count],
            size,
            order: Vec::new(),
            order_dirty: false,
            scratch: TickScratch::default(),
        }
    }

    pub fn insert(&mut self, pos: Pos, node: Node) -> Result<Option<Node>, GraphError> {
        if !self.contains_pos(&pos) {
            return Err(GraphError::PositionOutOfBounds {
                pos,
                size: self.size,
            });
        }

        let grid_index = self.grid_index(&pos);
        let node_id = self.grid[grid_index];
        if node_id != EMPTY_NODE {
            return Ok(Some(std::mem::replace(&mut self.nodes[node_id], node)));
        }

        let node_id = self.nodes.len();
        self.nodes.push(node);
        self.positions.push(pos);
        self.grid[grid_index] = node_id;
        self.order_dirty = true;
        Ok(None)
    }

    pub fn node(&self, pos: &Pos) -> Option<&Node> {
        self.node_id(pos).map(|node_id| &self.nodes[node_id])
    }

    pub fn node_mut(&mut self, pos: &Pos) -> Option<&mut Node> {
        self.node_id(pos).map(|node_id| &mut self.nodes[node_id])
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
        self.rebuild_order_if_dirty();
        self.evaluate_nodes(context);
        self.collect_outputs()?;
        self.reserve_routes();
        propagate_transport_blocks(&self.nodes, &mut self.scratch);
        self.restore_unrouted_outputs()?;
        self.commit_routes()?;
        Ok(())
    }

    fn evaluate_nodes(&mut self, context: &mut NodeContext) {
        for node_id in &self.order {
            self.nodes[*node_id].eval(context);
        }
    }

    fn collect_outputs(&mut self) -> Result<(), GraphError> {
        self.scratch.outputs.clear();
        for &from in &self.order {
            let pos = self.positions[from];
            let directions = self.nodes[from].output_directions();

            let mut routes = ArrayVec::<Route, MAX_OUTPUT_DIRECTIONS>::new();
            for direction in directions {
                if let Some(to) = self.neighbor_id(pos, direction) {
                    routes.push(Route { direction, to });
                }
            }

            let Some(item) = self.nodes[from].take_output() else {
                continue;
            };
            if routes.is_empty() {
                restore_output(&mut self.nodes[from], pos, item)?;
            } else {
                self.scratch.outputs.push(Output {
                    from,
                    routes,
                    item: Some(item),
                    route: None,
                });
            }
        }
        Ok(())
    }

    fn reserve_routes(&mut self) {
        self.scratch.remaining_capacity.clear();
        self.scratch
            .remaining_capacity
            .extend(self.nodes.iter().map(Node::input_capacity));
        for output in &mut self.scratch.outputs {
            for route in output.routes.iter() {
                if self.scratch.remaining_capacity[route.to] > 0 {
                    self.scratch.remaining_capacity[route.to] -= 1;
                    output.route = Some(*route);
                    break;
                }
            }
        }
    }

    fn restore_unrouted_outputs(&mut self) -> Result<(), GraphError> {
        for output in &mut self.scratch.outputs {
            if output.route.is_none() {
                let item = output
                    .item
                    .take()
                    .expect("unrouted outputs still own their items");
                restore_output(
                    &mut self.nodes[output.from],
                    self.positions[output.from],
                    item,
                )?;
            }
        }
        Ok(())
    }

    fn commit_routes(&mut self) -> Result<(), GraphError> {
        for output in &mut self.scratch.outputs {
            let Some(route) = output.route else {
                continue;
            };
            let item = output
                .item
                .take()
                .expect("routed outputs still own their items");
            if let Some(rejected) = self.nodes[route.to].accept(item) {
                restore_output(
                    &mut self.nodes[output.from],
                    self.positions[output.from],
                    rejected,
                )?;
            } else {
                self.nodes[output.from].confirm_output(route.direction);
            }
        }
        Ok(())
    }

    fn contains_pos(&self, pos: &Pos) -> bool {
        pos.x < self.size.0 && pos.y < self.size.1
    }

    fn grid_index(&self, pos: &Pos) -> usize {
        pos.y * self.size.0 + pos.x
    }

    fn node_id(&self, pos: &Pos) -> Option<NodeId> {
        if !self.contains_pos(pos) {
            return None;
        }
        let node_id = self.grid[self.grid_index(pos)];
        (node_id != EMPTY_NODE).then_some(node_id)
    }

    fn neighbor_id(&self, pos: Pos, direction: Direction) -> Option<NodeId> {
        let next = self.next_pos(&pos, direction)?;
        self.grid
            .get(self.grid_index(&next))
            .copied()
            .filter(|node_id| *node_id != EMPTY_NODE)
    }

    fn rebuild_order_if_dirty(&mut self) {
        if !self.order_dirty && self.order.len() == self.nodes.len() {
            return;
        }

        self.order.clear();
        self.order.extend(0..self.nodes.len());
        self.order.sort_unstable_by_key(|node_id| {
            let pos = self.positions[*node_id];
            (pos.y, pos.x)
        });
        self.order_dirty = false;
    }
}

#[derive(Debug, Clone)]
struct Output {
    from: NodeId,
    routes: ArrayVec<Route, MAX_OUTPUT_DIRECTIONS>,
    item: Option<crate::item::Item>,
    route: Option<Route>,
}

#[derive(Debug, Clone, Copy)]
struct Route {
    direction: Direction,
    to: NodeId,
}

#[derive(Debug, Default)]
struct TickScratch {
    outputs: Vec<Output>,
    remaining_capacity: Vec<usize>,
    incoming: Vec<ArrayVec<usize, MAX_INCOMING>>,
    blocked: Vec<bool>,
    queue: VecDeque<NodeId>,
}

fn propagate_transport_blocks(nodes: &[Node], scratch: &mut TickScratch) {
    if scratch.incoming.len() < nodes.len() {
        scratch
            .incoming
            .resize_with(nodes.len(), ArrayVec::<usize, MAX_INCOMING>::new);
    }
    for incoming in &mut scratch.incoming {
        incoming.clear();
    }
    scratch.blocked.clear();
    scratch.blocked.resize(nodes.len(), false);
    scratch.queue.clear();

    for (output_index, output) in scratch.outputs.iter().enumerate() {
        if let Some(route) = output.route {
            scratch.incoming[route.to]
                .try_push(output_index)
                .expect("a node cannot have more than four incoming routes");
        }
    }

    for output in &scratch.outputs {
        if output.route.is_none() && !scratch.blocked[output.from] {
            scratch.blocked[output.from] = true;
            scratch.queue.push_back(output.from);
        }
    }

    while let Some(blocked_node) = scratch.queue.pop_front() {
        if !nodes[blocked_node].is_transport() {
            continue;
        }

        for &incoming in &scratch.incoming[blocked_node] {
            let output = &mut scratch.outputs[incoming];
            if output.route.is_some_and(|route| route.to == blocked_node) {
                output.route = None;
                if !scratch.blocked[output.from] {
                    scratch.blocked[output.from] = true;
                    scratch.queue.push_back(output.from);
                }
            }
        }
    }
}

fn restore_output(node: &mut Node, pos: Pos, item: crate::item::Item) -> Result<(), GraphError> {
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

        assert_eq!(error, GraphError::PositionOutOfBounds { pos, size: (2, 1) });
    }
}
