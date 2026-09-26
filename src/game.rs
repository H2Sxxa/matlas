use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::NodeContext,
    entity::{
        atlas::Atlas,
        goal::Goals,
        machines::{MachineError, MachineKind, Machines},
        relics::Relics,
        repo::Repo,
        stats::Stats,
    },
    eval::graph::{Graph, GraphError},
    loot::{self, OFFER_COUNT, Reward, RewardOffer},
    node::{Direction, Node, Pos},
    rng::Rng,
    view::GameView,
};

// The machines a fresh run begins with.
//
// A run cannot bootstrap itself. Goals measure crafted items, revenue and
// discoveries, and none of those move while the grid is empty, so loot can never
// hand out the first machine. The kit is what turns an empty grid into a working
// factory: two generators feed belts into a mixer, and a belt carries the result
// into an inbound or a sell, depending on the goal.
pub const STARTING_KIT: [MachineKind; 9] = [
    MachineKind::Generator,
    MachineKind::Generator,
    MachineKind::Belt,
    MachineKind::Belt,
    MachineKind::Belt,
    MachineKind::Belt,
    MachineKind::Mixer,
    MachineKind::Inbound,
    MachineKind::Sell,
];

// Owns the factory and the meta systems around it. The goal chain, the relics and
// the machine stock live here so that claiming a reward can touch the graph and
// the RNG.
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub graph: Graph,
    pub context: NodeContext,
    pub goals: Goals,
    pub relics: Relics,
    pub machines: Machines,
    pub offers: Vec<RewardOffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimError {
    NoSuchOffer { index: usize, offered: usize },
    Machine(MachineError),
}

impl Display for ClaimError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimError::NoSuchOffer { index, offered } => {
                write!(
                    f,
                    "offer {index} is out of range: {offered} offers available"
                )
            }
            ClaimError::Machine(error) => write!(f, "machine reward rejected: {error}"),
        }
    }
}

impl Error for ClaimError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClaimError::Machine(error) => Some(error),
            ClaimError::NoSuchOffer { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceError {
    Machine(MachineError),
    Graph(GraphError),
}

impl Display for PlaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceError::Machine(error) => write!(f, "machine placement rejected: {error}"),
            PlaceError::Graph(error) => write!(f, "placement left the grid: {error:?}"),
        }
    }
}

impl Error for PlaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PlaceError::Machine(error) => Some(error),
            PlaceError::Graph(_) => None,
        }
    }
}

impl Game {
    // Starts a run with the base item already discovered, so goals have something
    // to measure against from the first tick.
    pub fn new(seed: [u8; 16], size: (usize, usize), machine_capacity: usize) -> Self {
        let mut rng = Rng::new(seed);
        let mut atlas = Atlas::new();
        atlas.init(&mut rng.generic);
        let stats = Stats::new();
        let goals = Goals::start(&atlas, &stats, &mut rng);
        Self {
            graph: Graph::new(size),
            context: NodeContext::new(atlas, Repo::new(), rng, stats),
            goals,
            relics: Relics::new(),
            machines: Machines::new(machine_capacity),
            offers: Vec::new(),
        }
    }

    pub fn tick(&mut self) -> Result<(), GraphError> {
        self.graph.tick(&mut self.context)?;
        self.refresh_goal();
        Ok(())
    }

    // Starts a run with a working factory instead of an empty grid.
    //
    // `extra_slots` is how many machines loot may still hand out beyond the kit, so
    // a run always starts with room for a few machine rewards.
    pub fn start(seed: [u8; 16], size: (usize, usize), extra_slots: usize) -> Self {
        let mut game = Self::new(seed, size, STARTING_KIT.len() + extra_slots);
        for kind in STARTING_KIT {
            game.machines
                .grant(kind)
                .expect("the kit fits the capacity it sizes");
        }
        game
    }

    // Snapshots the run for a presentation layer. The simulation is only borrowed,
    // so a renderer can call this as often as it likes.
    pub fn view(&self) -> GameView {
        GameView::build(self)
    }

    // Rolls reward offers once the current goal is met. Offers stay pending until
    // the player claims one, so this is a no-op while a choice is open.
    pub fn refresh_goal(&mut self) -> bool {
        if !self.offers.is_empty()
            || !self
                .goals
                .is_complete(&self.context.atlas, &self.context.stats)
        {
            return false;
        }
        self.offers = loot::roll_offers(&mut self.context.rng, &self.machines, OFFER_COUNT);
        true
    }

    // Applies one of the pending offers, then draws the next goal.
    pub fn claim(&mut self, index: usize) -> Result<Reward, ClaimError> {
        let chosen = self
            .offers
            .get(index)
            .cloned()
            .ok_or(ClaimError::NoSuchOffer {
                index,
                offered: self.offers.len(),
            })?;
        self.apply(&chosen.reward)?;
        self.offers.clear();
        self.goals.advance(
            &self.context.atlas,
            &self.context.stats,
            &mut self.context.rng,
        );
        Ok(chosen.reward)
    }

    pub fn apply(&mut self, reward: &Reward) -> Result<(), ClaimError> {
        match reward {
            Reward::Relic(relic) => {
                self.relics.add(relic.clone());
                self.relics.apply(&mut self.context.rng);
            }
            Reward::Machine(kind) => {
                self.machines.grant(*kind).map_err(ClaimError::Machine)?;
            }
            Reward::Graph(expansion) => {
                self.graph.grow(expansion.width, expansion.height);
            }
        }
        Ok(())
    }

    // Places a stock machine on the grid. A machine already at `pos` is returned to
    // stock, and the whole placement is rolled back when the position is unusable.
    pub fn place_machine(
        &mut self,
        kind: MachineKind,
        pos: Pos,
        direction: Direction,
    ) -> Result<Option<Node>, PlaceError> {
        self.machines.install(kind).map_err(PlaceError::Machine)?;
        match self.graph.insert(pos, kind.build(direction)) {
            Ok(replaced) => {
                if let Some(displaced) = replaced
                    .as_ref()
                    .and_then(|node| MachineKind::from_object(&node.object))
                {
                    self.machines.uninstall(displaced);
                }
                Ok(replaced)
            }
            Err(error) => {
                self.machines.uninstall(kind);
                Err(PlaceError::Graph(error))
            }
        }
    }

    // Removes the node at `pos`, returning a tracked machine to stock. Returns the
    // removed machine kind, or None when the cell was empty or held an untracked node.
    pub fn remove_machine(&mut self, pos: &Pos) -> Option<MachineKind> {
        let node = self.graph.remove(pos)?;
        let kind = MachineKind::from_object(&node.object);
        if let Some(kind) = kind {
            self.machines.uninstall(kind);
        }
        kind
    }
}

#[cfg(test)]
mod tests {
    use super::{ClaimError, Game, PlaceError};
    use crate::{
        entity::machines::{MachineError, MachineKind},
        entity::relics::Relic,
        eval::graph::GraphError,
        loot::{GraphExpansion, Reward},
        node::{Direction, Node, NodeObject, Pos, outbound::OutBoundNode},
        rng::RandomType,
    };

    #[test]
    fn placing_and_removing_machines_keeps_the_slots_honest() {
        let mut game = Game::new([1; 16], (3, 3), 2);
        game.machines
            .grant(MachineKind::Belt)
            .expect("slot is free");
        game.machines
            .grant(MachineKind::Mixer)
            .expect("slot is free");
        let pos = Pos { x: 0, y: 0 };

        assert!(
            game.place_machine(MachineKind::Belt, pos, Direction::Right)
                .expect("placement stays in bounds")
                .is_none()
        );
        assert_eq!(game.machines.stock(MachineKind::Belt), 0);
        assert_eq!(game.machines.free_slots(), 0);

        let replaced = game
            .place_machine(MachineKind::Mixer, pos, Direction::Right)
            .expect("placement stays in bounds")
            .expect("the belt is displaced");
        assert!(matches!(replaced.object, NodeObject::Transport(_)));
        assert_eq!(game.machines.stock(MachineKind::Belt), 1);
        assert_eq!(game.machines.stock(MachineKind::Mixer), 0);
        assert_eq!(game.machines.free_slots(), 0);

        assert_eq!(game.remove_machine(&pos), Some(MachineKind::Mixer));
        assert_eq!(game.machines.stock(MachineKind::Mixer), 1);
        assert!(game.remove_machine(&pos).is_none());
    }

    #[test]
    fn a_rejected_placement_leaves_the_stock_untouched() {
        let mut game = Game::new([1; 16], (3, 3), 1);
        game.machines
            .grant(MachineKind::Belt)
            .expect("slot is free");
        let outside = Pos { x: 3, y: 0 };

        let error = game
            .place_machine(MachineKind::Belt, outside, Direction::Right)
            .expect_err("the position is outside the grid");

        assert_eq!(
            error,
            PlaceError::Graph(GraphError::PositionOutOfBounds {
                pos: outside,
                size: (3, 3),
            })
        );
        assert_eq!(game.machines.stock(MachineKind::Belt), 1);
        assert_eq!(game.machines.placed(), 0);
    }

    #[test]
    fn rewards_are_applied_to_the_run() {
        let mut game = Game::new([1; 16], (3, 3), 1);

        game.apply(&Reward::Graph(GraphExpansion {
            width: 1,
            height: 2,
        }))
        .expect("graph rewards always apply");
        assert_eq!(game.graph.size(), (4, 5));

        game.apply(&Reward::Relic(Relic::new(
            "Test Relic".to_string(),
            RandomType::Loot,
            0.25,
        )))
        .expect("relic rewards always apply");
        assert_eq!(game.context.rng.luck(RandomType::Loot), 0.25);
        assert_eq!(game.relics.len(), 1);

        game.apply(&Reward::Machine(MachineKind::Belt))
            .expect("the only slot is free");
        assert_eq!(game.machines.stock(MachineKind::Belt), 1);
        assert_eq!(
            game.apply(&Reward::Machine(MachineKind::Mixer)),
            Err(ClaimError::Machine(MachineError::NoFreeSlot {
                capacity: 1,
                owned: 1,
            }))
        );
    }

    // A run has to survive a save and a load, which is what the front end keeps in
    // storage. Every node type has to round trip, because nodes cross as an
    // internally tagged enum and that representation is picky about its variants.
    #[test]
    fn a_run_round_trips_through_its_saved_form() {
        let mut game = Game::start([4; 16], (8, 4), 2);
        // The kit covers the basic loop; the routers only arrive as loot.
        for kind in [MachineKind::Distributor, MachineKind::Overflow] {
            game.machines
                .grant(kind)
                .expect("the run has a slot open for each router");
        }
        let layout = [
            (MachineKind::Generator, Direction::Right),
            (MachineKind::Belt, Direction::Right),
            (MachineKind::Mixer, Direction::Down),
            (MachineKind::Inbound, Direction::Left),
            (MachineKind::Distributor, Direction::Up),
            (MachineKind::Overflow, Direction::Right),
            (MachineKind::Sell, Direction::Down),
        ];
        for (index, (kind, direction)) in layout.into_iter().enumerate() {
            game.place_machine(kind, Pos { x: index, y: 0 }, direction)
                .expect("the kit machines cover the layout");
        }
        game.graph
            .insert(
                Pos { x: 0, y: 1 },
                Node::new(NodeObject::Outbound(OutBoundNode::new(0)), Direction::Right),
            )
            .expect("the outbound position is inside the grid");

        // Enough ticks to fill the buffers and let the mixer craft: a save has to
        // carry the grid, the node buffers and the recipe table.
        for _ in 0..6 {
            game.tick().expect("the layout stays inside the grid");
        }
        assert!(
            game.context.stats.crafted() > 0,
            "the layout crafts an item"
        );
        assert!(
            game.context.atlas.recipes().next().is_some(),
            "the layout records a recipe"
        );

        let saved = serde_json::to_string(&game).expect("a run is serialisable");
        let reloaded: Game = serde_json::from_str(&saved).expect("a save is loadable");

        assert_eq!(reloaded.view(), game.view());
    }
}
