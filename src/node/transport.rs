use serde::{Deserialize, Serialize};

use crate::{
    context::NodeContext,
    item::Item,
    node::{NodeBehavior, NodeObject},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportNode(Option<Item>);

impl NodeBehavior for TransportNode {
    const NAME: &'static str = "Transport";

    fn eval(&mut self, _context: &mut NodeContext) {
        // Transport Node has no evaluation logic, it simply holds an item for transport.
    }

    fn push(&mut self, _context: &mut NodeContext, _next: Option<NodeObject>) {
        // Transport Node just hold 1 item, how to push to anothoer node is not defined yet.
        todo!("Transport push");
    }
}
