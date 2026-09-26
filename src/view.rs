// Read-only projection of a run, built for presentation layers.
//
// A view is derived from a borrow of the game state, so building one never changes
// the run and a renderer can snapshot as often as it likes. Every type here is
// serialisable, so a view can cross a process or a language boundary as it is.
//
// Nodes keep their buffers private, so the projections below go through `NodeState`
// and the existing read-only accessors. A renderer must never take an output to
// inspect it: that mutates the simulation.

use serde::{Deserialize, Serialize};

use crate::{
    entity::{
        atlas::Atlas,
        goal::{GoalKind, Progress},
        machines::MachineKind,
        repo::Repo,
    },
    game::Game,
    item::ItemId,
    loot::RewardOffer,
    node::{Direction, MAX_INPUT_SLOTS, Node, NodeObject, Pos},
    rng::RandomType,
};

// The machine vocabulary a renderer draws with.
//
// `MachineKind` describes the loot pool, which deliberately leaves out outbound
// terminals, so the view needs its own vocabulary to name every node that can sit
// on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Generator,
    Belt,
    Mixer,
    Inbound,
    Outbound,
    Distributor,
    Overflow,
    Sell,
}

impl NodeKind {
    pub fn from_object(object: &NodeObject) -> Self {
        match object {
            NodeObject::Generator(_) => Self::Generator,
            NodeObject::Inbound(_) => Self::Inbound,
            NodeObject::Outbound(_) => Self::Outbound,
            NodeObject::Mixer(_) => Self::Mixer,
            NodeObject::Transport(_) => Self::Belt,
            NodeObject::Distributor(_) => Self::Distributor,
            NodeObject::Overflow(_) => Self::Overflow,
            NodeObject::Sell(_) => Self::Sell,
        }
    }
}

// One occupied cell as a renderer sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeView {
    pub pos: Pos,
    pub kind: NodeKind,
    pub direction: Direction,
    // The item buffered as this node's output, if any.
    pub held: Option<ItemId>,
    // Items parked in input slots. Only the mixer fills these.
    pub slots: [Option<ItemId>; MAX_INPUT_SLOTS],
    // Items queued for the next eval, for drains that take any number of items.
    pub queued: usize,
    // Delivery preference, nearest first: the next item leaves through the first
    // direction that can receive it.
    pub outputs: Vec<Direction>,
    // Free input slots, so a renderer can show a jam without reading the buffers.
    pub input_capacity: usize,
}

impl NodeView {
    pub fn from_node(node: &Node, pos: Pos) -> Self {
        let state = node.state();
        Self {
            pos,
            kind: NodeKind::from_object(&node.object),
            direction: node.direction,
            held: state.held.map(|item| item.id),
            slots: state.slots.map(|slot| slot.map(|item| item.id)),
            queued: state.queued,
            outputs: node.output_directions().into_iter().collect(),
            input_capacity: node.input_capacity(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemInfo {
    pub id: ItemId,
    pub name: String,
    pub level: usize,
    pub value: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeInfo {
    pub left: ItemId,
    pub right: ItemId,
    pub out: ItemId,
}

// Everything the atlas knows: the item codex and the recipes behind it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasView {
    pub items: Vec<ItemInfo>,
    pub recipes: Vec<RecipeInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoEntry {
    pub item: ItemId,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatsView {
    pub crafted: usize,
    pub revenue: usize,
}

// The current objective plus how far the factory has got with it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GoalView {
    pub kind: GoalKind,
    pub progress: Progress,
    pub completed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelicView {
    pub name: String,
    pub target: RandomType,
    pub luck: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockEntry {
    pub kind: MachineKind,
    pub count: usize,
}

// Every machine the run owns. Kinds with an empty stock stay in the list so a
// renderer can show a stable palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachinesView {
    pub capacity: usize,
    pub placed: usize,
    pub free_slots: usize,
    pub stock: Vec<StockEntry>,
}

// The whole run, flattened for a renderer. Pending offers are part of the view so a
// reward choice survives a page reload when the view is used as a save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameView {
    pub size: (usize, usize),
    pub nodes: Vec<NodeView>,
    pub atlas: AtlasView,
    pub repo: Vec<RepoEntry>,
    pub stats: StatsView,
    pub goal: GoalView,
    pub relics: Vec<RelicView>,
    pub machines: MachinesView,
    pub offers: Vec<RewardOffer>,
}

impl GameView {
    pub fn build(game: &Game) -> Self {
        Self {
            size: game.graph.size(),
            nodes: game
                .graph
                .iter()
                .map(|(pos, node)| NodeView::from_node(node, pos))
                .collect(),
            atlas: atlas_view(&game.context.atlas),
            repo: repo_view(&game.context.repo),
            stats: StatsView {
                crafted: game.context.stats.crafted(),
                revenue: game.context.stats.revenue(),
            },
            goal: GoalView {
                kind: game.goals.current(),
                progress: game
                    .goals
                    .progress(&game.context.atlas, &game.context.stats),
                completed: game.goals.completed(),
            },
            relics: game
                .relics
                .items()
                .iter()
                .map(|relic| RelicView {
                    name: relic.name.clone(),
                    target: relic.target,
                    luck: relic.luck,
                })
                .collect(),
            machines: MachinesView {
                capacity: game.machines.capacity(),
                placed: game.machines.placed(),
                free_slots: game.machines.free_slots(),
                stock: MachineKind::ALL
                    .into_iter()
                    .map(|kind| StockEntry {
                        kind,
                        count: game.machines.stock(kind),
                    })
                    .collect(),
            },
            offers: game.offers.clone(),
        }
    }
}

fn atlas_view(atlas: &Atlas) -> AtlasView {
    let mut items: Vec<ItemInfo> = atlas
        .ids()
        .map(|id| ItemInfo {
            id,
            name: atlas.safe_name(id),
            level: atlas.safe_level(id),
            value: atlas.safe_value(id),
        })
        .collect();
    items.sort_by_key(|item| item.id);

    let mut recipes: Vec<RecipeInfo> = atlas
        .recipes()
        .map(|((left, right), out)| RecipeInfo { left, right, out })
        .collect();
    recipes.sort_by_key(|recipe| (recipe.out, recipe.left, recipe.right));

    AtlasView { items, recipes }
}

fn repo_view(repo: &Repo) -> Vec<RepoEntry> {
    repo.entries()
        .map(|(item, count)| RepoEntry { item, count })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{GameView, NodeKind};
    use crate::{
        entity::machines::MachineKind,
        game::Game,
        node::{Direction, Pos},
    };

    // (0,0) Gv
    // (0,1) M>  (1,1) B>  (2,1) I
    // (0,2) G^
    // Legend: G generator | M mixer | B belt | I inbound | v ^ > facing.
    // A renderer has to see every step of the craft without touching the run: the
    // mixer slots while it waits, the belt while it carries, and the inbound queue
    // before the repository takes the item.
    #[test]
    fn a_view_shows_what_every_node_holds() {
        let mut game = Game::start([5; 16], (3, 3), 2);
        for (kind, pos, direction) in [
            (MachineKind::Generator, Pos { x: 0, y: 0 }, Direction::Down),
            (MachineKind::Mixer, Pos { x: 0, y: 1 }, Direction::Right),
            (MachineKind::Belt, Pos { x: 1, y: 1 }, Direction::Right),
            (MachineKind::Inbound, Pos { x: 2, y: 1 }, Direction::Right),
            (MachineKind::Generator, Pos { x: 0, y: 2 }, Direction::Up),
        ] {
            game.place_machine(kind, pos, direction)
                .expect("the kit machines fit the layout");
        }

        game.tick().expect("the layout stays inside the grid");
        let waiting = game.view();
        let mixer = node_at(&waiting, Pos { x: 0, y: 1 });
        assert_eq!(mixer.slots, [Some(0), Some(0)], "both sources are parked");
        assert_eq!(mixer.held, None, "the mixer crafts on its next eval");

        game.tick().expect("the layout stays inside the grid");
        let carrying = game.view();
        assert_eq!(
            node_at(&carrying, Pos { x: 1, y: 1 }).held,
            Some(1),
            "the belt carries the crafted item"
        );
        assert_eq!(carrying.stats.crafted, 1);
        assert!(
            carrying.atlas.items.iter().any(|item| item.id == 1),
            "the crafted item is in the codex"
        );

        game.tick().expect("the layout stays inside the grid");
        game.tick().expect("the layout stays inside the grid");
        let draining = game.view();
        assert_eq!(
            node_at(&draining, Pos { x: 2, y: 1 }).queued,
            1,
            "the inbound queue shows what the next eval will deposit"
        );

        game.tick().expect("the layout stays inside the grid");
        let delivered = game.view();
        // The pipeline is running at full speed: the mixer crafts on every eval, so
        // the repository takes one crafted item every two ticks.
        assert_eq!(
            delivered
                .repo
                .iter()
                .find(|entry| entry.item == 1)
                .map(|entry| entry.count),
            Some(2),
            "the crafted items reached the repository"
        );
    }

    // A renderer needs the whole grid in one walk, not one lookup per cell.
    #[test]
    fn a_view_walks_every_placed_node() {
        let mut game = Game::start([5; 16], (2, 2), 0);
        game.place_machine(MachineKind::Belt, Pos { x: 1, y: 1 }, Direction::Up)
            .expect("the position is inside the grid");

        let view: GameView = game.view();
        assert_eq!(view.size, (2, 2));
        assert_eq!(view.nodes.len(), 1);
        assert_eq!(view.nodes[0].pos, Pos { x: 1, y: 1 });
        assert_eq!(view.nodes[0].kind, NodeKind::Belt);
    }

    fn node_at(view: &GameView, pos: Pos) -> &super::NodeView {
        view.nodes
            .iter()
            .find(|node| node.pos == pos)
            .expect("the layout placed a node there")
    }
}
