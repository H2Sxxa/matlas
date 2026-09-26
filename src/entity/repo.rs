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
        let mut count = self.inner.entry_sync(item_id).or_insert(0);
        if *count == 0 {
            None
        } else {
            *count -= 1;
            Some(Item { id: item_id })
        }
    }

    pub fn count(&self, item_id: ItemId) -> Count {
        self.inner
            .get_sync(&item_id)
            .map_or(0, |count| *count.get())
    }
}
