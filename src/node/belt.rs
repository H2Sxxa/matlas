use serde::{Deserialize, Serialize};

use crate::{context::NodeContext, item::Item, node::NodeBehavior};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeltNode(Option<Item>);

impl BeltNode {
    pub fn new() -> Self {
        Self(None)
    }
}

impl Default for BeltNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for BeltNode {
    const NAME: &'static str = "Belt";

    fn eval(&mut self, _context: &mut NodeContext) {
        // Transport Node has no evaluation logic, it simply holds an item for transport.
    }

    fn take_output(&mut self) -> Option<Item> {
        self.0.take()
    }

    fn restore_output(&mut self, item: Item) -> Result<(), Item> {
        if self.0.is_none() {
            self.0 = Some(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    fn accept(&mut self, item: Item) -> Option<Item> {
        if self.0.is_none() {
            self.0 = Some(item);
            None
        } else {
            Some(item)
        }
    }

    fn input_capacity(&self) -> usize {
        usize::from(self.0.is_none())
    }
}
