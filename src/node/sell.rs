// Sink that turns delivered items into revenue.
//
// Like the inbound, it accepts any number of items and drains them on eval, but
// instead of storing them it prices each one, books the revenue and drops it.
use crate::{context::NodeContext, item::Item, node::NodeBehavior, rng::RandomType};
use serde::{Deserialize, Serialize};

// A sale pays between half and one and a half times the atlas value. The roll runs
// through the sell stream, so sell relics shift the average price up.
const PRICE_MIN: f64 = 0.5;
const PRICE_MAX: f64 = 1.5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellNode {
    ins: Vec<Item>,
}

impl SellNode {
    pub fn new() -> Self {
        Self { ins: Vec::new() }
    }
}

impl Default for SellNode {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehavior for SellNode {
    const NAME: &'static str = "Sell";

    fn eval(&mut self, context: &mut NodeContext) {
        while let Some(item) = self.ins.pop() {
            let value = context.atlas.safe_value(item.id);
            let factor = context
                .rng
                .roll_range(RandomType::Sell, PRICE_MIN..PRICE_MAX);
            context.stats.record_revenue(value as f64 * factor);
        }
    }

    fn accept(&mut self, item: Item) -> Option<Item> {
        self.ins.push(item);
        None
    }

    fn input_capacity(&self) -> usize {
        usize::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::SellNode;
    use crate::{
        item::Item,
        node::{Direction, NodeBehavior, Pos},
        rng::{Luck, RandomType},
        test_support::{context, generator, graph, place, sell, ticks},
    };

    // [queued: .] --accept(A), accept(B)--> [queued: A, B] --eval--> revenue
    // Legend: A first item | B second item; both are priced in the same eval.
    #[test]
    fn books_revenue_for_every_item_it_drains() {
        let mut node = SellNode::new();
        let mut context = context();
        context.atlas.init(&mut context.rng.generic);

        assert_eq!(node.input_capacity(), usize::MAX);
        assert!(node.accept(Item { id: 0 }).is_none());
        assert!(node.accept(Item { id: 0 }).is_none());

        node.eval(&mut context);

        // Two base items worth 1 each, priced in [0.5, 1.5], so 1..=3 in total.
        assert!((1..=3).contains(&context.stats.revenue()));
    }

    // (0,0) G>  (1,0) S
    // Legend: G generator | S sell | > facing right.
    #[test]
    fn sells_generated_items() {
        let mut graph = graph((2, 1));
        place(&mut graph, Pos { x: 0, y: 0 }, generator(Direction::Right));
        place(&mut graph, Pos { x: 1, y: 0 }, sell(Direction::Right));
        let mut context = context();

        ticks(&mut graph, &mut context, 3);

        assert!((1..=3).contains(&context.stats.revenue()));
    }

    // Relics that target the sell stream must actually raise income, so this
    // compares the same seeded stream with and without sell luck.
    #[test]
    fn sell_luck_raises_the_average_income() {
        let plain = average_income(0.0);
        let lucky = average_income(0.5);

        assert!(
            lucky > plain,
            "sell luck should raise income: plain={plain}, lucky={lucky}"
        );
    }

    fn average_income(luck: f64) -> f64 {
        let mut context = context();
        context
            .rng
            .set_luck(Luck::zero().biased(RandomType::Sell, luck));
        context.atlas.init(&mut context.rng.generic);
        let mut node = SellNode::new();

        let sales = 4_000;
        for _ in 0..sales {
            assert!(node.accept(Item { id: 0 }).is_none());
            node.eval(&mut context);
        }
        context.stats.revenue() as f64 / sales as f64
    }
}
