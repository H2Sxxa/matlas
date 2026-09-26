use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::{
    item::Item,
    node::{Direction, MAX_OUTPUT_DIRECTIONS, NodeBehavior, NodeState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributorNode {
    out: Option<Item>,
    next_output: u8,
}

impl DistributorNode {
    pub fn new() -> Self {
        Self {
            out: None,
            next_output: 0,
        }
    }
}

impl Default for DistributorNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for DistributorNode {
    const NAME: &'static str = "Distributor";

    fn state(&self) -> NodeState {
        NodeState {
            held: self.out.clone(),
            ..NodeState::default()
        }
    }

    fn take_output(&mut self) -> Option<Item> {
        self.out.take()
    }

    fn restore_output(&mut self, item: Item) -> Result<(), Item> {
        if self.out.is_none() {
            self.out = Some(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    fn accept(&mut self, item: Item) -> Option<Item> {
        if self.out.is_none() {
            self.out = Some(item);
            None
        } else {
            Some(item)
        }
    }

    fn input_capacity(&self) -> usize {
        usize::from(self.out.is_none())
    }

    fn output_directions(
        &self,
        direction: Direction,
    ) -> ArrayVec<Direction, MAX_OUTPUT_DIRECTIONS> {
        let outputs = [direction, direction.left(), direction.right()];
        let mut result = ArrayVec::new();
        for offset in 0..outputs.len() {
            result.push(outputs[(usize::from(self.next_output) + offset) % outputs.len()]);
        }
        result
    }

    fn confirm_output(&mut self, output_direction: Direction, facing: Direction) {
        let outputs = [facing, facing.left(), facing.right()];
        if let Some(index) = outputs
            .iter()
            .position(|candidate| *candidate == output_direction)
        {
            self.next_output = ((index + 1) % outputs.len()) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DistributorNode;
    use crate::{
        node::{Direction, NodeBehavior},
        test_support::{block_forward, context, distributor, routed_outputs, router_graph},
    };

    // facing right -> (right, up, down); after up was used -> (down, right, up)
    // Legend: outputs are listed first to last and the router tries them in order.
    #[test]
    fn rotates_the_output_order_after_every_delivery() {
        let mut node = DistributorNode::new();

        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Right, Direction::Up, Direction::Down]
        );

        node.confirm_output(Direction::Up, Direction::Right);
        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Down, Direction::Right, Direction::Up]
        );

        node.confirm_output(Direction::Down, Direction::Right);
        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Right, Direction::Up, Direction::Down]
        );
    }

    // (2,0) I
    // (2,1) B^
    // (1,2) G>  (2,2) D>  (3,2) B>  (4,2) I
    // (2,3) Bv
    // (2,4) I
    // Legend: G generator | D distributor | B belt | I inbound | > v ^ facing.
    // '.' means no output was loaded that tick, otherwise the loaded output.
    #[test]
    fn rotates_across_forward_left_and_right() {
        let mut graph = router_graph(distributor(Direction::Right));
        let mut context = context();

        assert_eq!(routed_outputs(&mut graph, &mut context, 6), ".FLRFL");
        assert_eq!(context.repo.count(0), 3);
    }

    // (2,0) I
    // (2,1) B^
    // (1,2) G>  (2,2) D>  (3,2) G  (4,2) I
    // (2,3) Bv
    // (2,4) I
    // Legend: G generator | D distributor | B belt | I inbound | > v ^ facing.
    // Forward is held by a generator that never frees up, so the rotation skips it.
    #[test]
    fn skips_a_blocked_output_and_keeps_rotating() {
        let mut graph = router_graph(distributor(Direction::Right));
        block_forward(&mut graph);
        let mut context = context();

        assert_eq!(routed_outputs(&mut graph, &mut context, 5), ".LRLR");
    }
}
