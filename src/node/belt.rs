use serde::{Deserialize, Serialize};

use crate::{context::NodeContext, item::Item, node::NodeBehavior};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeltNode(Option<Item>);

impl BeltNode {
    pub fn new() -> Self {
        Self(None)
    }
}

impl Default for BeltNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for BeltNode {
    const NAME: &'static str = "Belt";

    fn eval(&mut self, _context: &mut NodeContext) {
        // Transport Node has no evaluation logic, it simply holds an item for transport.
    }

    fn take_output(&mut self) -> Option<Item> {
        self.0.take()
    }

    fn restore_output(&mut self, item: Item) -> Result<(), Item> {
        if self.0.is_none() {
            self.0 = Some(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    fn accept(&mut self, item: Item) -> Option<Item> {
        if self.0.is_none() {
            self.0 = Some(item);
            None
        } else {
            Some(item)
        }
    }

    fn input_capacity(&self) -> usize {
        usize::from(self.0.is_none())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        item::Item,
        node::{Direction, NodeBehavior, Pos},
        test_support::{belt, context, graph, inbound, outbound, peek_output, place, ticks},
    };
    use super::BeltNode;

    // repository[7] = 1   (0,0) O>  (1,0) B>  (2,0) I
    // Legend: O outbound (item 7) | B belt | I inbound | > facing right.
    // The single item spends exactly one tick in the belt: the belt holds it after
    // the first tick, the inbound holds it after the second, and the repository
    // counts it again on the third.
    #[test]
    fn moves_its_item_on_the_next_tick() {
        let mut graph = graph((3, 1));
        let belt_pos = Pos { x: 1, y: 0 };
        place(&mut graph, Pos { x: 0, y: 0 }, outbound(7, Direction::Right));
        place(&mut graph, belt_pos, belt(Direction::Right));
        place(&mut graph, Pos { x: 2, y: 0 }, inbound(Direction::Right));
        let mut context = context();
        context.repo.inbound(7);

        ticks(&mut graph, &mut context, 1);
        assert_eq!(peek_output(&mut graph, belt_pos), Some(Item { id: 7 }));
        assert_eq!(context.repo.count(7), 0);

        ticks(&mut graph, &mut context, 1);
        assert_eq!(peek_output(&mut graph, belt_pos), None);
        assert_eq!(context.repo.count(7), 0);

        ticks(&mut graph, &mut context, 1);
        assert_eq!(context.repo.count(7), 1);
    }

    // [held: .] --accept(A)--> [held: A] --accept(B)--> [held: A], B comes back
    // Legend: A first item | B second item; a belt carries one item at a time.
    #[test]
    fn refuses_a_second_item_while_it_holds_one() {
        let mut belt = BeltNode::new();
        let first = Item { id: 0 };
        let second = Item { id: 1 };

        assert!(belt.accept(first.clone()).is_none());
        assert_eq!(belt.input_capacity(), 0);
        assert_eq!(belt.accept(second.clone()), Some(second));
        assert_eq!(belt.take_output(), Some(first.clone()));
        assert_eq!(belt.restore_output(first), Ok(()));
        assert_eq!(belt.restore_output(Item { id: 2 }), Err(Item { id: 2 }));
    }
}
