use crate::topic::simple::SimpleTopicRepository;
use crate::topic::TopicError;
use crate::topic::TopicError::TopicPatchFailed;
use crate::topic::TopicPatchError::MergeFailed;
use crate::topic::{Merge, Topic};
use futures::{future, Future, IntoFuture, Sink, Stream};
use std::path::PathBuf;

pub struct SimpleSubscription<State> {
    sink: Box<dyn Sink<SinkItem = State, SinkError = TopicError>>,
    stream: Box<dyn Stream<Item = State, Error = TopicError>>,
}

pub struct SimpleTopic<State> {
    id: String,
    state: State,
    subscription: Option<SimpleSubscription<State>>,
}

impl<State: Merge + Clone> SimpleTopic<State> {
    pub fn new(id: String, state: State) -> SimpleTopic<State> {
        SimpleTopic::<State> {
            id,
            state,
            subscription: None,
        }
    }

    fn apply_patch(&mut self, patch: State) -> Result<State, TopicError> {
        self.state
            .merge(patch)
            .map_err(|err| TopicPatchFailed(MergeFailed(err)))
    }
}

impl<State: Merge + Clone + 'static> Topic<State> for SimpleTopic<State> {
    type UnitFuture = Box<dyn Future<Item = (), Error = TopicError>>;
    type StateFuture = Box<dyn Future<Item = State, Error = TopicError>>;
    type UpdateStream = Box<dyn Stream<Item = State, Error = TopicError>>;

    fn snapshot(&self) -> Self::StateFuture {
        Box::new(future::ok(self.state.clone()))
    }

    fn patch(&mut self, patch: State) -> Self::UnitFuture {
        Box::new(
            self.apply_patch(patch)
                .map(|state| {
                    self.state = state;
                    ()
                })
                .into_future(),
        )
    }

    fn merge(&mut self, patch: State) -> Self::StateFuture {
        Box::new(
            self.apply_patch(patch)
                .map(|state| {
                    self.state = state.clone();
                    state
                })
                .into_future(),
        )
    }

    fn subscribe(&mut self) -> Self::UpdateStream {
        Box::new(future::ok(self.state.clone()).into_stream())
    }
}
