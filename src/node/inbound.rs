use crate::{context::NodeContext, item::Item, node::NodeBehavior};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InBoundNode {
    ins: Vec<Item>,
}

impl NodeBehavior for InBoundNode {
    const NAME: &'static str = "InBound";

    fn eval(&mut self, context: &mut NodeContext) {
        while let Some(item) = self.ins.pop() {
            context.repo.inbound(item.id);
        }
    }

    // Inbound has no output, so no push
}
