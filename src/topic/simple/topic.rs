use crate::topic::simple::SimpleTopicRepository;
use crate::topic::TopicAccessError::IOFailed;
use crate::topic::TopicError;
use crate::topic::TopicError::TopicAccessFailed;
use crate::topic::TopicError::TopicPatchFailed;
use crate::topic::TopicPatchError::MergeFailed;
use crate::topic::{Merge, Topic};
use futures::sink::Send;
use futures::stream::Once;
use futures::unsync::mpsc;
use futures::{future, Future, IntoFuture, Sink, Stream};
use futures::{stream, Async};
use std::io::Error;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use tokio::prelude::AsyncSink;
use tokio::sync::mpsc::channel;

pub struct SimpleSubscription<State> {
    sink: Box<dyn Sink<SinkItem = State, SinkError = TopicError>>,
}

pub struct InMemTopic<State> {
    id: String,
    state: State,
    subscription: Option<SimpleSubscription<State>>,
}

impl<State: Merge + Clone> InMemTopic<State> {
    pub fn new(id: String, state: State) -> InMemTopic<State> {
        InMemTopic::<State> {
            id,
            state,
            subscription: None,
        }
    }

    fn apply_patch(&mut self, patch: &State) -> Result<State, TopicError> {
        self.state
            .merge(patch)
            .map_err(|err| TopicPatchFailed(MergeFailed(err)))
    }
}

impl<State: Merge + Clone + 'static> Topic<State> for InMemTopic<State> {
    type TFS = InMemTopic<State>;
    type StateFuture = Box<dyn Future<Item = State, Error = TopicError>>;
    type TopicFuture = Box<dyn Future<Item = Self::TFS, Error = TopicError>>;
    type UpdateStream = Box<dyn Stream<Item = State, Error = TopicError>>;

    fn snapshot(&self) -> Self::StateFuture {
        Box::new(future::ok(self.state.clone()))
    }

    fn patch(mut self, patch: State) -> Self::TopicFuture {
        let patched = self.apply_patch(&patch).map(|state| {
            self.state = state;
        });

        let mut subscription: Option<SimpleSubscription<State>> = None;
        std::mem::swap(&mut self.subscription, &mut subscription);

        let publication = match subscription {
            Some(sub) => Box::new(sub.sink.send(patch).map(|sink| {
                self.subscription = Some(SimpleSubscription {
                    sink: Box::new(sink),
                });
                self
            })) as Self::TopicFuture,
            None => Box::new(future::ok(self)) as Self::TopicFuture,
        };

        let published = patched.into_future().and_then(move |_| publication);

        Box::new(published)
    }

    fn subscribe(&mut self) -> Self::UpdateStream {
        // TODO: This should improve, we simply drop any previous subscription
        // no warning, no way to restore it either :)
        if self.subscription.is_some() {
            warn!("Overwriting previous subscription");
        }

        let (mut sink, stream) = channel::<State>(20);

        // FIXME: this is 'fine' in the current implementation, but changes to the implementation
        // of channel could block thread progress beyond this code block.
        warn!("Blocking until current state dispatched to new subscription");
        while let Err(_) = sink.try_send(self.state.clone()) {
            sleep(Duration::from_millis(1))
        }

        self.subscription = Some(SimpleSubscription {
            sink: Box::new(sink.sink_from_err()),
        });

        Box::new(stream.from_err())
    }
}
