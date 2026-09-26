use crate::context::NodeContext;

pub mod generator;
pub mod inbound;
pub mod mixturer;
pub mod outbound;

pub trait Node<I, O> {
    const NAME: &'static str;
    fn eval(&mut self, context: &mut NodeContext, input: I) -> O;
}