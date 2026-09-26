// Generator will push Item / tick
use crate::{context::NodeContext, item::Item, node::NodeBehavior};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorNode {
    out: Option<Item>,
}

impl GeneratorNode {
    pub fn new() -> Self {
        Self { out: None }
    }
}

impl Default for GeneratorNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for GeneratorNode {
    const NAME: &'static str = "Generator";

    fn eval(&mut self, context: &mut NodeContext) {
        if self.out.is_none() {
            self.out = Some(context.atlas.base(&mut context.rng.generic));
        }
    }

    fn take_output(&mut self) -> Option<Item> {
        self.out.take()
    }

    fn restore_output(&mut self, item: Item) -> Result<(), Item> {
        if self.out.is_none() {
            self.out = Some(item);
            Ok(())
        } else {
            Err(item)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        item::Item,
        node::{Direction, Pos},
        test_support::{context, generator, graph, inbound, peek_output, place, ticks},
    };

    // (0,0) G>  (1,0) I
    // Legend: G generator | I inbound | > facing right.
    // One item leaves the generator per tick and lands in the repository one tick
    // after it arrives at the inbound.
    #[test]
    fn emits_one_item_per_tick() {
        let mut graph = graph((2, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, inbound(Direction::Right));
        let mut context = context();

        for (tick, expected) in [0, 1, 2].into_iter().enumerate() {
            ticks(&mut graph, &mut context, 1);
            assert_eq!(context.repo.count(0), expected, "after tick {}", tick + 1);
        }
    }

    // (0,0) G>  (1,0) .
    // Legend: G generator | . empty cell | > facing right, so there is no route.
    // A generator without a route holds its item instead of piling up new ones.
    #[test]
    fn keeps_its_item_while_it_has_no_route() {
        let mut graph = graph((2, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        let mut context = context();

        ticks(&mut graph, &mut context, 3);

        assert_eq!(context.repo.count(0), 0);
        assert_eq!(
            peek_output(&mut graph, Pos { x: 0, y: 0 }),
            Some(Item { id: 0 })
        );
    }
}
