use crate::{context::NodeContext, item::Item, node::Node};

pub struct GeneratorNode;

impl Node<(), Item> for GeneratorNode {
    const NAME: &'static str = "Generator";

    fn eval(&mut self, context: &mut NodeContext, _input: ()) -> Item {
        context.atlas.base(&mut context.rng.discover)
    }
}
