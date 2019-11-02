use crate::topic::Topic;
use futures::Future;

pub trait TopicFactory<TS> {
    type TTS: Topic<TS>;
    type CreateError;
    type TopicFuture: Future<Item = Self::TTS, Error = Self::CreateError>;

    fn create(&self, initial_state: TS) -> Self::TopicFuture;
}
