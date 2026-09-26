use serde::{Deserialize, Serialize};

use crate::{
    entity::{atlas::Atlas, repo::Repo},
    eval::graph::Graph,
    rng::Rng,
};

// Nodes Work on Context, Atlas and RNG should work isolated
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeContext {
    pub atlas: Atlas,
    pub repo: Repo,
    pub rng: Rng,
    pub graph: Graph,
}
