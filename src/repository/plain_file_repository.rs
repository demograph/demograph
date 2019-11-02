use std::io::Error;

use std::path::Path;
use std::path::PathBuf;

use futures;
use futures::future;

use futures::Future;
use futures::Sink;
use hyper::rt::Stream;
use hyper::Chunk;
use json_patch::merge;
use serde_json::Value;
use tokio::fs::OpenOptions;
use tokio::io::ErrorKind;
use tokio_codec::Decoder;

use crate::api::http::chunks_codec::ChunksCodec;
use crate::domain::TopicOld;

use super::error::*;
use super::flatten_sink::FlattenSinkOps;
use super::TopicRepository;

#[derive(Clone)]
pub struct PlainFileRepository {
    base_path: String,
}

impl PlainFileRepository {
    pub fn new(base_dir: &str) -> PlainFileRepository {
        PlainFileRepository {
            base_path: base_dir.to_owned(),
        }
    }

    pub fn topic_path(&self, name: &String) -> PathBuf {
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
    ) -> Box<dyn Future<Item = PlainFileTopic, Error = TopicRepositoryError> + Send> {
        let buf = self.topic_path(&name);
        Box::new(future::ok(PlainFileTopic::new(name, buf)))
    }

    fn reload(
        &self,
        name: String,
    ) -> Box<dyn Future<Item = PlainFileTopic, Error = TopicRepositoryError> + Send> {
        let path = self.topic_path(&name);
        Box::new(match path.exists() {
            true => future::ok(PlainFileTopic::new(name, path)),
            false => future::err(TopicRepositoryError::TopicLoadError(Error::from(
                ErrorKind::NotFound,
            ))),
        })
    }

    fn remove(
        &self,
        name: String,
    ) -> Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send> {
        let path = self.topic_path(&name);
        Box::new(match path.exists() {
            true => future::result(
                std::fs::remove_file(path).map_err(TopicRepositoryError::TopicRemovalError),
            ),
            false => future::err(TopicRepositoryError::TopicRemovalError(Error::from(
                ErrorKind::NotFound,
            ))),
        })
    }
}

#[derive(Clone)]
pub struct PlainFileTopic {
    name: String,
    path: PathBuf,
}

impl PlainFileTopic {
    pub fn new(name: String, path: PathBuf) -> PlainFileTopic {
        PlainFileTopic { name, path }
    }
}

impl TopicOld for PlainFileTopic {
    /** @deprecated Have to find an alternative still though */
    fn chunk_sink(
        &self,
    ) -> Box<dyn Sink<SinkItem = Chunk, SinkError = TopicRepositoryError> + Send> {
        let open_file = readwrite_options()
            .create(true)
            .truncate(true)
            .open(self.path.clone())
            .map_err(TopicRepositoryError::TopicLoadError);

        let file_sink = open_file.map(|file| {
            Decoder::framed(ChunksCodec::new(), file)
                .sink_map_err(TopicRepositoryError::TopicWriteError)
        });

        Box::new(file_sink.flatten_sink())
    }
    /** @deprecated Have to find an alternative still though */
    fn chunk_source(&self) -> Box<dyn Stream<Item = Chunk, Error = TopicRepositoryError> + Send> {
        let open_file = readwrite_options()
            .create(false)
            .open(self.path.clone())
            .map_err(TopicRepositoryError::TopicLoadError);

        let file_stream = open_file.map(|file| {
            Decoder::framed(ChunksCodec::new(), file).map_err(TopicRepositoryError::TopicReadError)
        });

        Box::new(file_stream.flatten_stream())
    }

    fn read_as_json(&self) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send> {
        let future = self.chunk_source().concat2().and_then(|bytes| {
            serde_json::from_slice::<Value>(&bytes)
                .map_err(TopicRepositoryError::TopicJsonReadError)
        });

        Box::new(future)
    }

    fn write_as_json(
        &self,
        patch: Value,
    ) -> Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send> {
        let maybe_chunk = serde_json::to_vec(&patch)
            .map(Chunk::from)
            .map_err(TopicRepositoryError::TopicJsonWriteError);

        match maybe_chunk {
            Ok(chunk) => Box::new(self.chunk_sink().send(chunk).map(|_| ())),
            Err(err) => Box::new(future::err(err)),
        }
    }

    fn merge_patch(
        &self,
        update: Value,
    ) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send> {
        // TODO: this is wrong, we shouldn't have to copy the (entire) context to reuse it later on
        // We are doing this to keep hold of the pathbuf, but in the process cloning multiple times
        // We could instead propagate the topic-name and generate the pathbuf at the required place
        debug!("merge_patch");
        let self_clone = (*self).clone();
        let post_update = self
            .read_as_json()
            .map(move |mut json| {
                merge(&mut json, &update);
                json
            })
            .and_then(move |json| {
                let copy = json.clone(); // TODO: Can this be done without clone? (check write side)
                self_clone.write_as_json(json).map(|_| copy)
            });

        Box::new(post_update)
    }
}

fn readwrite_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).read(true).truncate(false);
    return options;
}
