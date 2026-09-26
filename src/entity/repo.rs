use serde::{Deserialize, Serialize};

use crate::item::{Item, ItemId};
pub type Count = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    counts: Vec<Count>,
}

impl Repo {
    pub fn new() -> Self {
        Self { counts: Vec::new() }
    }

    pub fn inbound(&mut self, item_id: ItemId) {
        if item_id >= self.counts.len() {
            self.counts.resize(item_id + 1, 0);
        }
        self.counts[item_id] += 1;
    }

    pub fn outbound(&mut self, item_id: ItemId) -> Option<Item> {
        let count = self.counts.get_mut(item_id)?;
        if *count == 0 {
            None
        } else {
            *count -= 1;
            Some(Item { id: item_id })
        }
    }

    pub fn count(&self, item_id: ItemId) -> Count {
        self.counts.get(item_id).copied().unwrap_or(0)
    }
}

impl Default for Repo {
    fn default() -> Self {
        Self::new()
    }
}
