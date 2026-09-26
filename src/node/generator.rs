// Generator will push Item / tick
use crate::{context::NodeContext, item::Item, node::NodeBehavior};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorNode {
    out: Option<Item>,
}

impl GeneratorNode {
    pub fn new() -> Self {
        Self { out: None }
    }
}

impl Default for GeneratorNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for GeneratorNode {
    const NAME: &'static str = "Generator";

    fn eval(&mut self, context: &mut NodeContext) {
        if self.out.is_none() {
            self.out = Some(context.atlas.base(&mut context.rng.generic));
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
