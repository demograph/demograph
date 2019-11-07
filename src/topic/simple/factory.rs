use crate::topic::simple::InMemTopic;
use crate::topic::{Merge, TopicFactory};
use futures::{future, Future};
use std::marker::PhantomData;

pub struct VolatileTopicFactory<TS> {
    _phantom: PhantomData<TS>,
}

impl<TS> VolatileTopicFactory<TS> {
    pub fn new() -> VolatileTopicFactory<TS> {
        VolatileTopicFactory::<TS> {
            _phantom: PhantomData,
        }
    }
}

impl<TS: Merge + Send + Clone + 'static> TopicFactory<TS> for VolatileTopicFactory<TS> {
    type TTS = InMemTopic<TS>;
    type CreateError = ();
    type TopicFuture = Box<dyn Future<Item = Self::TTS, Error = Self::CreateError> + Send>;

    fn create(&self, initial_state: TS) -> Self::TopicFuture {
        Box::new(future::ok(InMemTopic::<TS>::new(
            String::from(""),
            initial_state,
        )))
    }
}
