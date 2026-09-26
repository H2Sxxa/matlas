use std::time::Instant;

use matlas::{
    context::NodeContext,
    entity::{atlas::Atlas, repo::Repo, stats::Stats},
    eval::graph::Graph,
    item::ItemId,
    node::{Direction, Node, NodeObject, Pos, belt::BeltNode, outbound::OutBoundNode},
    rng::Rng,
};

const DEFAULT_BELT_COUNT: usize = 50_000;
const DEFAULT_MEASURED_TICKS: usize = 100;

fn main() {
    let belt_count = std::env::args()
        .nth(1)
        .map(|value| value.parse().expect("belt count must be a usize"))
        .unwrap_or(DEFAULT_BELT_COUNT);
    let measured_ticks = std::env::args()
        .nth(2)
        .map(|value| value.parse().expect("tick count must be a usize"))
        .unwrap_or(DEFAULT_MEASURED_TICKS);

    let mut graph = Graph::new((belt_count, 2));
    for belt_index in 0..belt_count {
        graph
            .insert(
                Pos {
                    x: belt_index,
                    y: 0,
                },
                Node::new(NodeObject::Transport(BeltNode::new()), Direction::Right),
            )
            .expect("belt positions stay inside the benchmark graph");
        graph
            .insert(
                Pos {
                    x: belt_index,
                    y: 1,
                },
                Node::new(
                    NodeObject::Outbound(OutBoundNode::new(belt_index)),
                    Direction::Up,
                ),
            )
            .expect("outbound positions stay inside the benchmark graph");
    }

    let mut context = NodeContext::new(Atlas::new(), Repo::new(), Rng::new([0; 16]), Stats::new());
    for item_id in 0..belt_count {
        context.repo.inbound(item_id as ItemId);
    }

    graph
        .tick(&mut context)
        .expect("benchmark graph stays inside its bounds");

    let started = Instant::now();
    for _ in 0..measured_ticks {
        graph
            .tick(&mut context)
            .expect("benchmark graph stays inside its bounds");
    }
    let elapsed = started.elapsed();

    println!(
        "belts={belt_count} ticks={measured_ticks} elapsed_ms={:.3} per_tick_us={:.3}",
        elapsed.as_secs_f64() * 1_000.0,
        elapsed.as_secs_f64() * 1_000_000.0 / measured_ticks as f64
    );
}
