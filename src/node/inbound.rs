use crate::{context::NodeContext, item::Item, node::NodeBehavior};
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

    fn accept(&mut self, item: Item) -> Option<Item> {
        self.ins.push(item);
        None
    }

    fn input_capacity(&self) -> usize {
        usize::MAX
    }
}
