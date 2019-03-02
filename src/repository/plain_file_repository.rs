use crate::domain::Topic;
use futures;
use futures::future;
use futures::Future;

use std::path::Path;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio_codec::Decoder;

use super::error::*;
use super::TopicRepository;
use futures::Sink;
use hyper::rt::Stream;
use hyper::Chunk;

use super::flatten_sink::FlattenSinkOps;
use crate::api::http::chunks_codec::ChunksCodec;
use std::io::Error;
use tokio::io::ErrorKind;

pub struct PlainFileRepository {
    base_path: String,
}

impl PlainFileRepository {
    pub fn new(base_dir: &str) -> PlainFileRepository {
        PlainFileRepository {
            base_path: base_dir.to_owned(),
        }
    }

    fn topic_path(&self, name: &String) -> PathBuf {
        Path::new(&self.base_path).join(Path::new(&(name.to_owned() + ".log")))
    }

    //    fn load_topic(
    //        &self,
    //        name: String,
    //        open_options: &OpenOptions,
    //    ) -> Box<Future<Item = PlainFileTopic, Error = TopicRepositoryError> + Send + 'static> {
    //        )
    //    }
}

impl TopicRepository for PlainFileRepository {
    type Topic = PlainFileTopic;

    fn load(
        &self,
        name: String,
    ) -> Box<Future<Item = PlainFileTopic, Error = TopicRepositoryError> + Send> {
        let buf = self.topic_path(&name);
        Box::new(future::ok(PlainFileTopic::new(name, buf)))
    }

    fn reload(
        &self,
        name: String,
    ) -> Box<Future<Item = PlainFileTopic, Error = TopicRepositoryError> + Send> {
        let path = self.topic_path(&name);
        Box::new(match path.exists() {
            true => future::ok(PlainFileTopic::new(name, path)),
            false => future::err(TopicRepositoryError::TopicLoadError(Error::from(
                ErrorKind::NotFound,
            ))),
        })
    }
}

impl Clone for PlainFileRepository {
    fn clone(&self) -> Self {
        PlainFileRepository {
            base_path: self.base_path.clone(),
        }
    }
}

pub struct PlainFileTopic {
    name: String,
    path: PathBuf,
}

impl PlainFileTopic {
    pub fn new(name: String, path: PathBuf) -> PlainFileTopic {
        PlainFileTopic { name, path }
    }
}

impl Topic for PlainFileTopic {
    /** @deprecated Have to find an alternative still though */
    fn chunk_sink(&self) -> Box<Sink<SinkItem = Chunk, SinkError = TopicRepositoryError> + Send> {
        let open_file = readwrite_options()
            .create(true)
            .open(self.path.clone())
            .map_err(TopicRepositoryError::TopicLoadError);

        let file_sink = open_file.map(|file| {
            Decoder::framed(ChunksCodec::new(), file)
                .sink_map_err(TopicRepositoryError::TopicWriteError)
        });

        Box::new(file_sink.flatten_sink())
    }
    /** @deprecated Have to find an alternative still though */
    fn chunk_source(&self) -> Box<Stream<Item = Chunk, Error = TopicRepositoryError> + Send> {
        let open_file = readwrite_options()
            .create(false)
            .open(self.path.clone())
            .map_err(TopicRepositoryError::TopicLoadError);

        let file_stream = open_file.map(|file| {
            Decoder::framed(ChunksCodec::new(), file).map_err(TopicRepositoryError::TopicReadError)
        });

        Box::new(file_stream.flatten_stream())
    }
}

fn readwrite_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).read(true).truncate(false);
    return options;
}

//struct DelayedSink<S: Sink<SinkItem = _, SinkError = _>>(S);
//
//impl<S> DelayedSink<S> {
//    fn unbuffered() -> impl Sink<SinkItem = _, SinkError = _>
//    where
//        W: Future<Sink<SinkItem = I, SinkError = E>>,
//    {
//
//    }
//}
//
//impl<W, SI, SE> Sink for DelayedSink<W>
//where
//    W: Future<Item = Sink<SinkItem = SI, SinkError = SE>, Error = SE>,
//{
//    type SinkItem = SI;
//    type SinkError = SE;
//
//    fn start_send(
//        &mut self,
//        item: Self::SinkItem,
//    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
//        unimplemented!()
//    }
//
//    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
//        unimplemented!()
//    }
//
//    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
//        unimplemented!()
//    }
//}
