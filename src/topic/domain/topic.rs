use futures::Future;
use futures::Stream;

use crate::topic::TopicError;
use std::fmt::Debug;

pub trait Topic<State> {
    type TFS: Topic<State>;
    type StateFuture: Future<Item = State, Error = TopicError> + Send;
    type TopicFuture: Future<Item = Self::TFS, Error = TopicError> + Send;
    type UpdateStream: Stream<Item = State, Error = TopicError> + Send;

    fn snapshot(&self) -> Self::StateFuture;

    fn patch(mut self, patch: State) -> Self::TopicFuture;

    fn subscribe(&mut self) -> Self::UpdateStream;
}

pub trait Merge
where
    Self: std::marker::Sized,
{
    type MergeError: Debug;

    fn merge(&self, patch: &Self) -> Result<Self, Self::MergeError>;
}
