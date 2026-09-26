use crate::{
    entity::{atlas::Atlas, repo::Repo},
    rng::Rng,
};

// Nodes Work on Context, Atlas and RNG should work isolated

#[derive(Debug)]
pub struct NodeContext {
    pub atlas: Atlas,
    pub repo: Repo,
    pub rng: Rng,
}
