use serde::{Deserialize, Serialize};

use crate::{context::NodeContext, item::Item, node::NodeBehavior};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MixturerNode {
    in0: Option<Item>,
    in1: Option<Item>,
    out: Option<Item>,
}

impl MixturerNode {
    pub fn new() -> Self {
        Self {
            in0: None,
            in1: None,
            out: None,
        }
    }
}

impl Default for MixturerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for MixturerNode {
    const NAME: &'static str = "Mixturer";

    fn eval(&mut self, context: &mut NodeContext) {
        if self.out.is_some() || self.in0.is_none() || self.in1.is_none() {
            return;
        }

        let a = self.in0.take().expect("mixer input 0 was checked");
        let b = self.in1.take().expect("mixer input 1 was checked");

        self.out = Some(context.atlas.mix(&mut context.rng.discover, &(a, b)))
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
        if self.in0.is_none() {
            self.in0 = Some(item);
            None
        } else if self.in1.is_none() {
            self.in1 = Some(item);
            None
        } else {
            Some(item)
        }
    }

    fn input_capacity(&self) -> usize {
        usize::from(self.in0.is_none()) + usize::from(self.in1.is_none())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        item::Item,
        node::{Direction, NodeBehavior, Pos},
        test_support::{belt, context, generator, graph, inbound, mixer, outbound, place, ticks},
    };
    use super::MixturerNode;

    // in0: 0 | in1: 0 | out: .   --eval-->   in0: . | in1: . | out: 0+0
    // Legend: in0/in1 input slots | out output buffer. One craft consumes one item
    // per slot and produces a single item on the same tick.
    #[test]
    fn crafts_from_both_slots_and_frees_them() {
        let mut node = MixturerNode::new();
        let mut context = context();
        let item = Item { id: 0 };

        assert_eq!(node.input_capacity(), 2);
        assert!(node.accept(item.clone()).is_none());
        assert_eq!(node.input_capacity(), 1);
        assert!(node.accept(item).is_none());
        assert_eq!(node.input_capacity(), 0);

        node.eval(&mut context);

        assert_eq!(node.take_output(), Some(Item { id: 1 }));
        assert_eq!(node.input_capacity(), 2);
    }

    // in0: 0 | in1: .   --eval-->   no output, in0 keeps its item
    // Legend: a craft needs both slots filled.
    #[test]
    fn waits_for_the_second_slot() {
        let mut node = MixturerNode::new();
        let mut context = context();

        assert!(node.accept(Item { id: 0 }).is_none());
        node.eval(&mut context);

        assert_eq!(node.take_output(), None);
        assert_eq!(node.input_capacity(), 1);
    }

    // in0: 0 | in1: 0   --accept(2)-->   2 comes back untouched
    // Legend: both slots taken means a third item is refused.
    #[test]
    fn refuses_inputs_while_both_slots_are_occupied() {
        let mut node = MixturerNode::new();
        let item = Item { id: 0 };

        assert!(node.accept(item.clone()).is_none());
        assert!(node.accept(item).is_none());
        assert_eq!(node.accept(Item { id: 2 }), Some(Item { id: 2 }));
        assert_eq!(node.input_capacity(), 0);
    }

    // (0,0) G>  (1,0) M>  (2,0) I
    // Legend: G generator | M mixer | I inbound | > facing right.
    // The generator fills in0, then in1, and the mixer then crafts item 1.
    #[test]
    fn fills_both_slots_from_one_generator() {
        let mut graph = graph((3, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, mixer(Direction::Right));
        place(&mut graph, Pos { x: 2, y: 0 }, inbound(Direction::Right));
        let mut context = context();

        ticks(&mut graph, &mut context, 4);

        assert_eq!(context.repo.count(0), 0);
        assert_eq!(context.repo.count(1), 1);
    }

    // (0,0) O>  (1,0) M>  (2,0) I
    // (1,2) G^  (1,1) B^
    // Legend: O outbound (item 0 taken from the repository) | M mixer | I inbound
    //         B belt | G generator | > ^ facing.
    // Both inputs reach the mixer on different ticks and still produce item 1.
    #[test]
    fn crafts_with_inputs_that_arrive_on_different_ticks() {
        let mut graph = graph((3, 3));
        place(&mut graph, Pos { x: 0, y: 0 }, outbound(0, Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, mixer(Direction::Right));
        place(&mut graph, Pos { x: 2, y: 0 }, inbound(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 1 }, belt(Direction::Up));
        place(&mut graph, Pos { x: 1, y: 2 }, generator(Direction::Up));
        let mut context = context();
        context.repo.inbound(0);

        ticks(&mut graph, &mut context, 4);

        assert_eq!(context.repo.count(0), 0);
        assert_eq!(context.repo.count(1), 1);
    }

    // (0,0) Gv
    // (0,1) M>  (1,1) I
    // (0,2) G^
    // Legend: G generator | M mixer | I inbound | v ^ > facing.
    // One source per slot; the crafted item carries atlas material metadata.
    #[test]
    fn registers_a_recipe_with_material_metadata() {
        let mut graph = graph((2, 3));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Down));
        place(&mut graph, Pos { x: 0, y: 1 }, mixer(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 1 }, inbound(Direction::Right));
        place(&mut graph, Pos { x: 0, y: 2 }, generator(Direction::Up));
        let mut context = context();

        ticks(&mut graph, &mut context, 3);

        assert_eq!(context.repo.count(1), 1);
        assert!(!context.atlas.safe_name(1).is_empty());
        assert!(context.atlas.safe_level(1) >= 1);
        assert!(context.atlas.safe_value(1) >= 1);
    }
}
