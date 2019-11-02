use crate::repository::TopicRepositoryError;
use crate::topic::Topic;
use futures::Future;

pub trait TopicRepository<TS> {
    type TTS: Topic<TS>;
    type TopicFuture: Future<Item = Self::TTS, Error = TopicRepositoryError>;
    type DeleteFuture: Future<Item = (), Error = TopicRepositoryError>;

    fn save(&self, id: String, topic: Self::TTS) -> Self::TopicFuture;

    fn delete(&self, id: String, topic: Self::TTS) -> Self::DeleteFuture;

    fn load(&self, id: String) -> Self::TopicFuture;
}
