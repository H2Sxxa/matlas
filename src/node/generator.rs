// Generator will push Item / tick
use crate::{context::NodeContext, item::Item, node::NodeBehavior};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorNode {
    out: Item,
}
