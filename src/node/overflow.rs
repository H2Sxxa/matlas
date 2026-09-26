use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::{
    item::Item,
    node::{Direction, MAX_OUTPUT_DIRECTIONS, NodeBehavior},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverflowNode {
    out: Option<Item>,
    next_overflow: bool,
}

impl OverflowNode {
    pub fn new() -> Self {
        Self {
            out: None,
            next_overflow: false,
        }
    }
}

impl Default for OverflowNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for OverflowNode {
    const NAME: &'static str = "Overflow";

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
        let sides = if self.next_overflow {
            [direction.right(), direction.left()]
        } else {
            [direction.left(), direction.right()]
        };
        let mut result = ArrayVec::new();
        result.push(direction);
        result.push(sides[0]);
        result.push(sides[1]);
        result
    }

    fn confirm_output(&mut self, output_direction: Direction, facing: Direction) {
        if output_direction == facing {
            return;
        }
        self.next_overflow = !self.next_overflow;
    }
}

#[cfg(test)]
mod tests {
    use super::OverflowNode;
    use crate::{
        node::{Direction, NodeBehavior},
        test_support::{
            block_forward, context, free_forward, overflow, routed_outputs, router_graph,
        },
    };

    // facing right -> (right, up, down); after up was used -> (right, down, up)
    // Legend: forward stays first, and only a side delivery flips the side order.
    #[test]
    fn keeps_forward_first_and_alternates_the_sides() {
        let mut node = OverflowNode::new();

        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Right, Direction::Up, Direction::Down]
        );

        node.confirm_output(Direction::Right, Direction::Right);
        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Right, Direction::Up, Direction::Down]
        );

        node.confirm_output(Direction::Up, Direction::Right);
        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Right, Direction::Down, Direction::Up]
        );

        node.confirm_output(Direction::Down, Direction::Right);
        assert_eq!(
            node.output_directions(Direction::Right).as_slice(),
            &[Direction::Right, Direction::Up, Direction::Down]
        );
    }

    // (2,0) I
    // (2,1) B^
    // (1,2) G>  (2,2) V>  (3,2) B>  (4,2) I
    // (2,3) Bv
    // (2,4) I
    // Legend: G generator | V overflow | B belt | I inbound | > v ^ facing.
    // '.' means no output was loaded that tick, otherwise the loaded output.
    // While forward drains, the sides stay untouched.
    #[test]
    fn prefers_forward_while_it_accepts() {
        let mut graph = router_graph(overflow(Direction::Right));
        let mut context = context();

        assert_eq!(routed_outputs(&mut graph, &mut context, 6), ".FFFFF");
    }

    // (2,0) I
    // (2,1) B^
    // (1,2) G>  (2,2) V>  (3,2) G  (4,2) I
    // (2,3) Bv
    // (2,4) I
    // Legend: G generator | V overflow | B belt | I inbound | > v ^ facing.
    // Forward is held by a generator that never frees up, so the overflow spills to
    // the sides and flips sides after every spill.
    #[test]
    fn alternates_sides_while_forward_is_blocked() {
        let mut graph = router_graph(overflow(Direction::Right));
        block_forward(&mut graph);
        let mut context = context();

        assert_eq!(routed_outputs(&mut graph, &mut context, 5), ".LRLR");
    }

    // (2,0) I
    // (2,1) B^
    // (1,2) G>  (2,2) V>  (3,2) G -> B>  (4,2) I
    // (2,3) Bv
    // (2,4) I
    // Legend: G generator | V overflow | B belt | I inbound | > v ^ facing.
    // After the blocked forward is replaced by a belt, forward takes over again.
    #[test]
    fn returns_to_forward_once_it_is_free_again() {
        let mut graph = router_graph(overflow(Direction::Right));
        block_forward(&mut graph);
        let mut context = context();

        assert_eq!(routed_outputs(&mut graph, &mut context, 2), ".L");

        free_forward(&mut graph);
        assert_eq!(routed_outputs(&mut graph, &mut context, 3), "FFF");
    }
}
