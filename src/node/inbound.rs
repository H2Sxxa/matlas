use crate::{context::NodeContext, item::Item, node::Node};

pub struct InBoundNode;

impl Node<Item, ()> for InBoundNode {
    const NAME: &'static str = "InBound";

    fn eval(&mut self, context: &mut NodeContext, input: Item) -> () {
        context.repo.inbound(input.id);
    }
}
