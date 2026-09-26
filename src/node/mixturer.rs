use serde::{Deserialize, Serialize};

use crate::{context::NodeContext, item::Item, node::{NodeBehavior, NodeObject}};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MixturerNode {
    in0: Option<Item>,
    in1: Option<Item>,
    out: Option<Item>,
}

impl NodeBehavior for MixturerNode {
    const NAME: &'static str = "Mixturer";

    fn eval(&mut self, context: &mut NodeContext) {
        if self.in0.is_none() || self.in1.is_none() {
            return;
        }

        let a = self.in0.take().unwrap();
        let b = self.in1.take().unwrap();

        self.out = Some(context.atlas.mix(&mut context.rng.discover, &(a, b)))
    }

    fn push(&mut self, _context: &mut NodeContext, _next: Option<NodeObject>) {
        todo!("Mixturer push");
    }
}
