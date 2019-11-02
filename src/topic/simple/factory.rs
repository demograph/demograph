use crate::topic::simple::SimpleTopic;
use crate::topic::{Merge, TopicFactory};
use futures::{future, Future};
use std::marker::PhantomData;

pub struct SimpleTopicFactory<TS> {
    _phantom: PhantomData<TS>,
}

impl<TS> SimpleTopicFactory<TS> {
    pub fn new() -> SimpleTopicFactory<TS> {
        SimpleTopicFactory::<TS> {
            _phantom: PhantomData,
        }
    }
}

impl<TS: Merge + Clone + 'static> TopicFactory<TS> for SimpleTopicFactory<TS> {
    type TTS = SimpleTopic<TS>;
    type CreateError = ();
    type TopicFuture = Box<dyn Future<Item = Self::TTS, Error = Self::CreateError>>;

    fn create(&self, initial_state: TS) -> Self::TopicFuture {
        Box::new(future::ok(SimpleTopic::<TS>::new(
            String::from(""),
            initial_state,
        )))
    }
}
