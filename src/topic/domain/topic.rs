use futures::Future;
use futures::Stream;

use crate::topic::MergeError;
use crate::topic::TopicError;

pub trait Topic<State> {
    type UnitFuture: Future<Item = (), Error = TopicError>;
    type StateFuture: Future<Item = State, Error = TopicError>;
    type UpdateStream: Stream<Item = State, Error = TopicError>;

    fn snapshot(&self) -> Self::StateFuture;

    fn patch(&mut self, patch: State) -> Self::UnitFuture;

    fn merge(&mut self, patch: State) -> Self::StateFuture;

    fn subscribe(&mut self) -> Self::UpdateStream;
}

pub trait Merge
where
    Self: std::marker::Sized,
{
    fn merge(&self, patch: Self) -> Result<Self, MergeError>;
}
