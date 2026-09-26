use serde::{Deserialize, Serialize};

use crate::{
    context::NodeContext, item::{Item, ItemId}, node::{NodeBehavior, NodeObject},
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutBoundNode {
    outid: ItemId,
    out: Option<Item>,
}

impl NodeBehavior for OutBoundNode {
    const NAME: &'static str = "OutBound";

    fn eval(&mut self, context: &mut NodeContext) {
        if let None = self.out
            && let Some(item) = context.repo.outbound(self.outid)
        {
            self.out = Some(item);
        }
    }

    fn push(&mut self, _context: &mut NodeContext, _next: Option<NodeObject>) {
        todo!()
    }
}
