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
