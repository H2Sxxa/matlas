use serde::{Deserialize, Serialize};

use crate::{
    context::NodeContext,
    item::{Item, ItemId},
    node::NodeBehavior,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutBoundNode {
    outid: ItemId,
    out: Option<Item>,
}

impl OutBoundNode {
    pub fn new(item_id: ItemId) -> Self {
        Self {
            outid: item_id,
            out: None,
        }
    }
}

impl NodeBehavior for OutBoundNode {
    const NAME: &'static str = "OutBound";

    fn eval(&mut self, context: &mut NodeContext) {
        if self.out.is_none()
            && let Some(item) = context.repo.outbound(self.outid)
        {
            self.out = Some(item);
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
}

#[cfg(test)]
mod tests {
    use super::OutBoundNode;
    use crate::{
        item::Item,
        node::{Direction, NodeBehavior, Pos},
        test_support::{context, graph, outbound, peek_output, place, ticks},
    };

    // repository[7] = 2, repository[8] = 1
    // [out: .] --eval--> [out: 7] --take--> [out: .] --eval--> [out: 7]
    // Legend: out output buffer; eval withdraws only while the buffer is empty.
    #[test]
    fn withdraws_one_item_while_its_output_buffer_is_free() {
        let mut node = OutBoundNode::new(7);
        let mut context = context();
        context.repo.inbound(7);
        context.repo.inbound(7);
        context.repo.inbound(8);

        node.eval(&mut context);
        assert_eq!(context.repo.count(7), 1);
        assert!(node.take_output().is_some());

        node.eval(&mut context);
        assert_eq!(context.repo.count(7), 0);
        assert!(node.take_output().is_some());

        node.eval(&mut context);
        assert_eq!(context.repo.count(7), 0);
        assert!(node.take_output().is_none());
        assert_eq!(context.repo.count(8), 1);
    }

    // repository[7] = 2   (0,0) O>  (1,0) .
    // Legend: O outbound | . empty cell | > facing right, so there is no route.
    // The withdrawn item cannot leave, so the node keeps it and stops withdrawing.
    #[test]
    fn holds_its_withdrawal_until_the_output_can_move() {
        let mut graph = graph((2, 1));
        place(
            &mut graph,
            Pos { x: 0, y: 0 },
            outbound(7, Direction::Right),
        );
        let mut context = context();
        context.repo.inbound(7);
        context.repo.inbound(7);

        ticks(&mut graph, &mut context, 3);

        assert_eq!(context.repo.count(7), 1);
        assert_eq!(
            peek_output(&mut graph, Pos { x: 0, y: 0 }),
            Some(Item { id: 7 })
        );
    }
}
