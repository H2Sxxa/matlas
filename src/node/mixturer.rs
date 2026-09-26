use serde::{Deserialize, Serialize};

use crate::{context::NodeContext, item::Item, node::NodeBehavior};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MixturerNode {
    in0: Option<Item>,
    in1: Option<Item>,
    out: Option<Item>,
}

impl MixturerNode {
    pub fn new() -> Self {
        Self {
            in0: None,
            in1: None,
            out: None,
        }
    }
}

impl Default for MixturerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for MixturerNode {
    const NAME: &'static str = "Mixturer";

    fn eval(&mut self, context: &mut NodeContext) {
        if self.out.is_some() || self.in0.is_none() || self.in1.is_none() {
            return;
        }

        let a = self.in0.take().expect("mixer input 0 was checked");
        let b = self.in1.take().expect("mixer input 1 was checked");

        self.out = Some(context.atlas.mix(&mut context.rng.discover, &(a, b)))
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

    fn accept(&mut self, item: Item) -> Option<Item> {
        if self.in0.is_none() {
            self.in0 = Some(item);
            None
        } else if self.in1.is_none() {
            self.in1 = Some(item);
            None
        } else {
            Some(item)
        }
    }

    fn input_capacity(&self) -> usize {
        usize::from(self.in0.is_none()) + usize::from(self.in1.is_none())
    }
}
