use scc::HashMap;
use serde::{Deserialize, Serialize};

use crate::item::{Item, ItemId};
pub type Count = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    inner: HashMap<ItemId, Count>,
}

impl Repo {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn inbound(&self, item_id: ItemId) {
        self.inner
            .entry_sync(item_id)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn outbound(&self, item_id: ItemId) -> Option<Item> {
        if let Some(count) = self.inner.get_sync(&item_id) {
            if *count > 0 {
                self.inner
                    .entry_sync(item_id)
                    .and_modify(|v| *v -= 1)
                    .or_insert(0);
                return Some(Item { id: item_id });
            }
        }
        None
    }
}
