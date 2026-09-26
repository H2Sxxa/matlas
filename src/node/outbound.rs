use crate::{
    context::NodeContext,
    item::{Item, ItemId},
    node::Node,
};

pub struct OutBoundNode {
    output: ItemId,
}

impl Node<(), Option<Item>> for OutBoundNode {
    const NAME: &'static str = "OutBound";

    fn eval(&mut self, context: &mut NodeContext, _input: ()) -> Option<Item> {
        context.repo.outbound(self.output)
    }
}
