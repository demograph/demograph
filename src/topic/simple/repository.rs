use crate::api::http::chunks_codec::ChunksCodec;
use crate::repository::flatten_sink::FlattenSinkOps;
use crate::topic::simple::InMemTopic;
use crate::topic::TopicRepositoryError;
use crate::topic::TopicRepositoryError::TopicSaveFailed;
use crate::topic::TopicSaveError;
use crate::topic::TopicSaveError::TopicFailed;
use crate::topic::{Merge, Topic, TopicRepository};
use futures::{future, Future};
use hyper::Chunk;
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tokio::codec::{Decoder, Encoder, Framed, FramedWrite};
use tokio::fs::OpenOptions;
use tokio::prelude::*;

pub struct SimpleTopicRepository<TS> {
    base_dir: &'static Path,
    _phantom: PhantomData<TS>,
}

impl<TS> SimpleTopicRepository<TS> {
    pub fn new(base_dir: &'static Path) -> SimpleTopicRepository<TS> {
        SimpleTopicRepository {
            base_dir,
            _phantom: PhantomData,
        }
    }

    pub fn topic_path(&self, id: &String) -> PathBuf {
        Path::new(&self.base_dir).join(Path::new(&(id.to_owned() + ".log")))
    }
}

impl<TS: Merge + Send + Clone + 'static> TopicRepository<TS> for SimpleTopicRepository<TS> {
    type TTS = InMemTopic<TS>;
    type TopicFuture = Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send>;
    type DeleteFuture = Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send>;

    fn save<Enc>(&self, id: String, topic: &mut Self::TTS, mut encoder: Enc) -> Self::TopicFuture
    where
        Enc: Encoder<Item = TS, Error = io::Error> + Send + 'static,
    {
        let open_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.topic_path(&id))
            .map_err(|err| TopicSaveFailed("file open error", TopicSaveError::IOFailed(err)));

        let downstream = open_file.map(move |file| {
            FramedWrite::new(file, encoder).sink_map_err(|err| {
                TopicSaveFailed("downstream error", TopicSaveError::IOFailed(err))
            })
        });

        let upstream = topic
            .subscribe()
            .map_err(|e| TopicSaveFailed("upstream error", TopicSaveError::TopicFailed(e)));

        let stream = upstream
            .forward(downstream.flatten_sink())
            .map(|(_stream, _sink)| ());

        Box::new(stream)
    }

    fn delete(&self, id: String, topic: Self::TTS) -> Self::DeleteFuture {
        unimplemented!()
    }

    fn load<Dec>(&self, id: String, decoder: Dec) -> Self::TopicFuture
    where
        Dec: Decoder + Send,
    {
        unimplemented!()
    }
}
