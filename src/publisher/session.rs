use std::net::SocketAddr;
use std::path::Path;
use std::prelude::v1::Vec;

use futures::future;
use futures::Sink;
use hyper::rt::{Future, Stream};
use hyper::service::Service;
use hyper::Chunk;
use hyper::{Body, Method, Request, Response, StatusCode};
use tokio::fs::OpenOptions;
use tokio_fs::File;

use crate::api::http::ResponseStream;
use crate::api::http::ResponseStreamError;
use crate::chunks_codec::ChunksCodec;
use crate::publisher::error::PublisherError;
use crate::LOG_DIR;

pub struct PublisherSession {
    remote: SocketAddr,
}

impl PublisherSession {
    pub fn new(remote: SocketAddr) -> PublisherSession {
        PublisherSession { remote }
    }
}

impl Service for PublisherSession {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = PublisherError;
    type Future = Box<Future<Item = Response<Body>, Error = PublisherError> + Send>;

    fn call(&mut self, req: Request<<Self as Service>::ReqBody>) -> <Self as Service>::Future {
        let mut response = Response::new(Body::empty());

        // TODO: Figure out how to extract the next two statements into a function Request<T> -> Vec<&str>
        let path: String = req.uri().path().to_owned();
        let segments: Vec<&str> = path
            .split("/")
            .filter(|segment| !segment.is_empty())
            .collect();

        debug!("Path segments: {:?}", segments);

        match (req.method(), &segments[..]) {
            (&Method::GET, []) => *response.body_mut() = Body::from("Try POSTing data to /publish"),
            (&Method::GET, ["topic", topic]) => {
                *response.body_mut() = Body::from(format!("{{\"topic\":\"{}\"}}", topic))
            }
            (&Method::POST, ["topic", topic]) => {
                self.handle_topic_publish(topic, req, &mut response)
            }
            _ => *response.status_mut() = StatusCode::NOT_FOUND,
        }

        Box::new(future::ok(response))
    }
}

impl PublisherSession {
    fn handle_topic_publish(
        &self,
        topic: &str,
        req: Request<<PublisherSession as Service>::ReqBody>,
        response: &mut Response<Body>,
    ) {
        debug!("Incoming connection from {}", self.remote);
        let chunk_source = req.into_body().map_err(PublisherError::BodyAccessError);
        let pipe = open_topic_log(topic)
            .map(create_chunk_sink)
            .and_then(|chunk_sink| chunk_source.forward(chunk_sink));
        // Some coercion required to use the future result as a result body
        let response_future: ResponseStream = Box::new(
            pipe.into_stream()
                .inspect_err(|e| error!("Failed to process request. {}", e))
                .map(|_| Chunk::from("{}"))
                .map_err(|e| Box::new(e) as ResponseStreamError),
        );
        *response.body_mut() = Body::from(response_future);
    }
}

fn open_topic_log(topic: &str) -> impl Future<Item = File, Error = PublisherError> {
    let path = Path::new(LOG_DIR).join(Path::new(&(topic.to_owned() + ".log")));

    OpenOptions::new()
        .write(true)
        .create(true)
        .read(false)
        .truncate(false)
        .open(path)
        .map_err(PublisherError::LogOpeningError)
}

fn create_chunk_sink(file: File) -> impl Sink<SinkItem = Chunk, SinkError = PublisherError> {
    tokio_codec::Decoder::framed(ChunksCodec::new(), file)
        .sink_map_err(PublisherError::LogWriteError)
}
