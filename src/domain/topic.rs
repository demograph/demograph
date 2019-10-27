use crate::repository::TopicRepositoryError;
use futures::Sink;
use futures::Stream;
use hyper::Chunk;

pub trait Topic {
    /** @deprecated Have to find an alternative still though */
    fn chunk_sink(
        &self,
    ) -> Box<dyn Sink<SinkItem = Chunk, SinkError = TopicRepositoryError> + Send>;
    /** @deprecated Have to find an alternative still though */
    fn chunk_source(&self) -> Box<dyn Stream<Item = Chunk, Error = TopicRepositoryError> + Send>;
}
