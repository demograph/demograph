use chunks_codec::ChunksCodec;
use futures::future;
use futures::Sink;
use hyper::rt::{Future, Stream};
use hyper::service::Service;
use hyper::Chunk;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::net::SocketAddr;
use tokio_fs::File;

use api::http::ResponseStream;
use api::http::ResponseStreamError;
use publisher::error::PublisherError;
use std::path::Path;
use tokio::fs::OpenOptions;
use LOG_DIR;

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

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                *response.body_mut() = Body::from("Try POSTing data to /publish");
            }
            (&Method::POST, "/publish") => {
                debug!("Incoming connection from {}", self.remote);

                let chunk_source = req.into_body().map_err(PublisherError::BodyAccessError);
                let pipe = open_session_log(self.remote)
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
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        };

        Box::new(future::ok(response))
    }
}

fn open_session_log(remote: SocketAddr) -> impl Future<Item = File, Error = PublisherError> {
    let path = Path::new(LOG_DIR).join(Path::new(&(remote.to_string() + ".log")));

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
