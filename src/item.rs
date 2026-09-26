use std::fmt::Display;

use serde::{Deserialize, Serialize};
pub type ItemId = usize;

// Data Object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: ItemId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalizedItem {
    pub name: String,
    pub item: Item,
    pub value: usize,
    pub level: usize,
}

impl Display for CapitalizedItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} Lv.{}",
            self.name[0..1].to_uppercase() + &self.name[1..],
            self.level
        ))
    }
}
