use crate::domain::Topic;
use crate::storage::error::TopicStorageError;
use crate::LOG_DIR;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio_codec::Decoder;
use tokio_fs::File;

use futures::future;
use futures::Sink;
use hyper::rt::{Future, Stream};
use hyper::Chunk;

use crate::api::http::chunks_codec::ChunksCodec; // <-- FIXME!

pub struct TopicStore {
    base_path: String,
}

impl TopicStore {
    pub fn new(base_dir: &str) -> TopicStore {
        TopicStore {
            base_path: base_dir.to_owned(),
        }
    }

    pub fn sink(&self, from: &Topic) -> impl Sink<SinkItem = Chunk, SinkError = TopicStorageError> {
        let file = OVERWRITE_OPTIONS
            .open(self.log_path(from))
            .map_err(TopicStorageError::TopicOpeningError);

        Decoder::framed(ChunksCodec::new(), file).sink_map_err(TopicStorageError::TopicWriteError)
    }

    pub fn source(&self, to: &Topic) -> impl Stream<Item = Chunk, Error = TopicStorageError> {
        let file = READONLY_OPTIONS
            .open(self.log_path(to))
            .map_err(TopicStorageError::TopicOpeningError);

        Decoder::framed(ChunksCodec::new(), file).map_err(TopicStorageError::TopicReadError)
    }

    fn log_path(&self, topic: &Topic) -> PathBuf {
        Path::new(self.base_path).join(Path::new(&(topic.name + ".log")))
    }
}

//trait StreamingStorage {
//    fn sink(from: Self) -> impl Sink<SinkItem = Chunk, SinkError = TopicStorageError>;
//    fn source(to: Self) -> impl Stream<Item = Chunk, Error = TopicStorageError>;
//}
//
//impl StreamingStorage {
//    fn new<T>(base_dir: &str) -> impl StreamingStorage<T> {
//
//    }
//
//    fn log_path(topic: &str) -> PathBuf {
//        Path::new(LOG_DIR).join(Path::new(&(topic.to_owned() + ".log")))
//    }
//
//    fn open_topic_log_overwrite(topic: &str) -> impl Future<Item = File, Error = TopicStorageError> {
//        OpenOptions::new()
//            .write(true)
//            .create(true)
//            .read(false)
//            .truncate(false)
//            .open(log_path(topic))
//            .map_err(TopicStorageError::TopicOpeningError)
//    }
//
//    fn create_chunk_sink(file: File) -> impl Sink<SinkItem = Chunk, SinkError = TopicStorageError> {
//        Decoder::framed(ChunksCodec::new(), file).sink_map_err(TopicStorageError::TopicWriteError)
//    }
//
//    fn open_topic_log_read(topic: &str) -> impl Future<Item = File, Error = TopicStorageError> {
//        OpenOptions::new()
//            .write(false)
//            .create(false)
//            .read(true)
//            .truncate(false)
//            .open(log_path(topic))
//            .map_err(TopicStorageError::TopicOpeningError)
//    }
//
//    fn create_chunk_source(file: File) -> impl Stream<Item = Chunk, Error = TopicStorageError> {
//        tokio_codec::Decoder::framed(ChunksCodec::new(), file).map_err(TopicStorageError::TopicReadError)
//    }
//}
