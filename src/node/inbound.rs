use crate::{
    context::NodeContext,
    item::Item,
    node::{NodeBehavior, NodeState},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InBoundNode {
    ins: Vec<Item>,
}

impl InBoundNode {
    pub fn new() -> Self {
        Self { ins: Vec::new() }
    }
}

impl Default for InBoundNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for InBoundNode {
    const NAME: &'static str = "InBound";

    fn eval(&mut self, context: &mut NodeContext) {
        while let Some(item) = self.ins.pop() {
            context.repo.inbound(item.id);
        }
    }

    fn state(&self) -> NodeState {
        NodeState {
            queued: self.ins.len(),
            ..NodeState::default()
        }
    }

    fn accept(&mut self, item: Item) -> Option<Item> {
        self.ins.push(item);
        None
    }

    fn input_capacity(&self) -> usize {
        usize::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::InBoundNode;
    use crate::{
        item::Item,
        node::{Direction, NodeBehavior, Pos},
        test_support::{context, generator, graph, inbound, place, ticks},
    };

    // [queued: .] --accept(A), accept(B)--> [queued: A, B] --eval--> repository
    // Legend: A first item | B second item; an inbound accepts any number of items.
    #[test]
    fn accepts_any_number_of_items_and_stores_them_on_eval() {
        let mut node = InBoundNode::new();
        let mut context = context();

        assert_eq!(node.input_capacity(), usize::MAX);
        assert!(node.accept(Item { id: 0 }).is_none());
        assert!(node.accept(Item { id: 1 }).is_none());

        node.eval(&mut context);

        assert_eq!(context.repo.count(0), 1);
        assert_eq!(context.repo.count(1), 1);
    }

    // (0,0) G>  (1,0) I
    // Legend: G generator | I inbound | > facing right.
    // Items land in the repository on the tick after they arrive.
    #[test]
    fn stores_delivered_items_on_the_following_tick() {
        let mut graph = graph((2, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, inbound(Direction::Right));
        let mut context = context();

        ticks(&mut graph, &mut context, 1);
        assert_eq!(context.repo.count(0), 0);

        ticks(&mut graph, &mut context, 1);
        assert_eq!(context.repo.count(0), 1);

        ticks(&mut graph, &mut context, 1);
        assert_eq!(context.repo.count(0), 2);
    }
}
