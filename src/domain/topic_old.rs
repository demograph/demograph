use futures::Stream;
use futures::{Future, Sink};
use hyper::Chunk;

use serde_json::Value;

use crate::repository::TopicRepositoryError;

pub trait TopicOld {
    /** @deprecated Have to find an alternative still though */
    fn chunk_sink(
        &self,
    ) -> Box<dyn Sink<SinkItem = Chunk, SinkError = TopicRepositoryError> + Send>;
    /** @deprecated Have to find an alternative still though */
    fn chunk_source(&self) -> Box<dyn Stream<Item = Chunk, Error = TopicRepositoryError> + Send>;

    fn read_as_json(&self) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send>;

    fn write_as_json(
        &self,
        patch: Value,
    ) -> Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send>;

    fn merge_patch(
        &self,
        patch: Value,
    ) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send>;
}
