use crate::{context::NodeContext, item::Item, node::Node};

pub struct MixturerNode {
    a: Option<Item>,
    b: Option<Item>,
}

impl Node<Item, Option<Item>> for MixturerNode {
    const NAME: &'static str = "Mixturer";

    fn eval(&mut self, context: &mut NodeContext, input: Item) -> Option<Item> {
        if self.a.is_none() {
            self.a = Some(input);
            return None;
        }

        if self.b.is_none() {
            return Some(
                context
                    .atlas
                    .mix(&mut context.rng.discover, &(self.a.take().unwrap(), input)),
            );
        }

        let a = self.a.take().unwrap();
        let b = self.b.take().unwrap();

        Some(context.atlas.mix(&mut context.rng.discover, &(a, b)))
    }
}
