// Shared fixtures for the node and graph tests.
//
// Every layout comment in the tests reads like the grid it describes: the first
// row is y = 0 and the first column is x = 0, so a node at (1, 0) sits one step
// right of (0, 0).
//
// Legend:
//   G generator | B belt (transport) | M mixer | I inbound | O outbound
//   D distributor | V overflow | . empty cell
//   > v < ^ node facing (right, down, up, left)

use crate::{
    context::NodeContext,
    entity::{atlas::Atlas, repo::Repo},
    eval::graph::Graph,
    item::{Item, ItemId},
    node::{
        Direction, Node, NodeObject, Pos, belt::BeltNode, distributor::DistributorNode,
        generator::GeneratorNode, inbound::InBoundNode, mixturer::MixturerNode,
        outbound::OutBoundNode, overflow::OverflowNode,
    },
    rng::Rng,
};

pub fn context() -> NodeContext {
    NodeContext::new(Atlas::new(), Repo::new(), Rng::new([0; 16]))
}

pub fn graph(size: (usize, usize)) -> Graph {
    Graph::new(size)
}

pub fn place(graph: &mut Graph, pos: Pos, node: Node) {
    graph
        .insert(pos, node)
        .expect("test layouts stay inside the graph");
}

pub fn generator(direction: Direction) -> Node {
    Node::new(NodeObject::Generator(GeneratorNode::new()), direction)
}

pub fn belt(direction: Direction) -> Node {
    Node::new(NodeObject::Transport(BeltNode::new()), direction)
}

pub fn mixer(direction: Direction) -> Node {
    Node::new(NodeObject::Mixer(MixturerNode::new()), direction)
}

pub fn inbound(direction: Direction) -> Node {
    Node::new(NodeObject::Inbound(InBoundNode::new()), direction)
}

pub fn outbound(item_id: ItemId, direction: Direction) -> Node {
    Node::new(NodeObject::Outbound(OutBoundNode::new(item_id)), direction)
}

pub fn distributor(direction: Direction) -> Node {
    Node::new(NodeObject::Distributor(DistributorNode::new()), direction)
}

pub fn overflow(direction: Direction) -> Node {
    Node::new(NodeObject::Overflow(OverflowNode::new()), direction)
}

pub fn tick(graph: &mut Graph, context: &mut NodeContext) {
    graph
        .tick(context)
        .expect("test graphs stay inside their bounds");
}

pub fn ticks(graph: &mut Graph, context: &mut NodeContext, count: usize) {
    for _ in 0..count {
        tick(graph, context);
    }
}

// Reads the item a node holds without changing it: the output is taken and put
// straight back, which is lossless for every node that buffers one output.
pub fn peek_output(graph: &mut Graph, pos: Pos) -> Option<Item> {
    let node = graph.node_mut(&pos).expect("peeked positions exist");
    let item = node.take_output()?;
    node.restore_output(item.clone())
        .expect("peeking puts back the item it took");
    Some(item)
}

// Router layout shared by the distributor and overflow tests:
//
//   (2,0) I
//   (2,1) B^         <- left output, drains up into (2,0)
//   (1,2) G>  (2,2) D or V>  (3,2) B>  (4,2) I   <- forward output
//   (2,3) Bv         <- right output, drains down into (2,4)
//   (2,4) I
//
// The router faces right, so forward is (3, 2), left is (2, 1) and right is
// (2, 3). Each output is a belt draining into an inbound, so all three are empty
// at the start of every tick and a delivery stays visible for exactly one tick.
pub const ROUTER: Pos = Pos { x: 2, y: 2 };
pub const FORWARD: Pos = Pos { x: 3, y: 2 };
pub const LEFT: Pos = Pos { x: 2, y: 1 };
pub const RIGHT: Pos = Pos { x: 2, y: 3 };

pub fn router_graph(router: Node) -> Graph {
    let mut graph = graph((5, 5));
    place(&mut graph, Pos { x: 1, y: 2 }, generator(Direction::Right));
    place(&mut graph, ROUTER, router);
    draining_output(&mut graph, FORWARD, Direction::Right, Pos { x: 4, y: 2 });
    draining_output(&mut graph, LEFT, Direction::Up, Pos { x: 2, y: 0 });
    draining_output(&mut graph, RIGHT, Direction::Down, Pos { x: 2, y: 4 });
    graph
}

fn draining_output(graph: &mut Graph, belt_pos: Pos, direction: Direction, sink: Pos) {
    place(graph, belt_pos, belt(direction));
    place(graph, sink, inbound(Direction::Right));
}

// Replaces the forward output with a generator that cannot reach a neighbor: it
// holds on to its item, so its input capacity stays at zero and the router never
// sees forward as free.
pub fn block_forward(graph: &mut Graph) {
    let replaced = graph
        .insert(FORWARD, generator(Direction::Up))
        .expect("forward stays inside the graph");
    assert!(replaced.is_some(), "forward starts out occupied");
}

pub fn free_forward(graph: &mut Graph) {
    let replaced = graph
        .insert(FORWARD, belt(Direction::Right))
        .expect("forward stays inside the graph");
    assert!(replaced.is_some(), "forward starts out occupied");
}

// Reports which output carried an item during the tick that just ran:
//   F forward | L left of forward | R right of forward | . nothing moved
pub fn routed_output(graph: &mut Graph, context: &mut NodeContext) -> char {
    tick(graph, context);
    for (pos, label) in [(FORWARD, 'F'), (LEFT, 'L'), (RIGHT, 'R')] {
        if output_belt_holds_item(graph, pos) {
            return label;
        }
    }
    '.'
}

pub fn routed_outputs(graph: &mut Graph, context: &mut NodeContext, count: usize) -> String {
    let mut observed = String::new();
    for _ in 0..count {
        observed.push(routed_output(graph, context));
    }
    observed
}

fn output_belt_holds_item(graph: &mut Graph, pos: Pos) -> bool {
    let is_belt = graph
        .node(&pos)
        .is_some_and(|node| matches!(node.object, NodeObject::Transport(_)));
    is_belt && peek_output(graph, pos).is_some()
}
