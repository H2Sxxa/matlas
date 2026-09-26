use serde::{Deserialize, Serialize};

use crate::{
    entity::{atlas::Atlas, repo::Repo, stats::Stats},
    rng::Rng,
};

// Shared services used by node evaluation. The graph is owned by its caller.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeContext {
    pub atlas: Atlas,
    pub repo: Repo,
    pub rng: Rng,
    pub stats: Stats,
}

impl NodeContext {
    pub fn new(atlas: Atlas, repo: Repo, rng: Rng, stats: Stats) -> Self {
        Self {
            atlas,
            repo,
            rng,
            stats,
        }
    }
}
