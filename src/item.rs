use serde::{Deserialize, Serialize};
pub type ItemId = usize;

// Data Object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: ItemId,
}


#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct CapitalizedItem {
    pub name: String,
    pub item: Item,
    pub value: usize,
    pub level: usize,
}