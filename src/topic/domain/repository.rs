use crate::topic::Topic;
use crate::topic::TopicRepositoryError;
use futures::Future;
use std::io;
use tokio::codec::Encoder;
use tokio_io::_tokio_codec::Decoder;

pub trait TopicRepository<TS> {
    type TTS: Topic<TS>;
    type TopicFuture: Future<Item = (), Error = TopicRepositoryError>;
    type DeleteFuture: Future<Item = (), Error = TopicRepositoryError>;

    fn save<Enc>(&self, id: String, topic: &mut Self::TTS, mut encoder: Enc) -> Self::TopicFuture
    where
        Enc: Encoder<Item = TS, Error = io::Error> + 'static;

    fn delete(&self, id: String, topic: Self::TTS) -> Self::DeleteFuture;

    fn load<Dec>(&self, id: String, decoder: Dec) -> Self::TopicFuture
    where
        Dec: Decoder;
}
