use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::prelude::v1::Vec;

use futures::future;
use futures::stream;
use futures::stream::AndThen;
use futures::IntoStream;
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
use crate::publisher::error::UserApiError;
use crate::LOG_DIR;

pub struct UserApiSession {
    remote: SocketAddr,
}

impl UserApiSession {
    pub fn new(remote: SocketAddr) -> UserApiSession {
        UserApiSession { remote }
    }
}

impl Service for UserApiSession {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = UserApiError;
    type Future = Box<Future<Item = Response<Body>, Error = UserApiError> + Send>;

    fn call(&mut self, req: Request<<Self as Service>::ReqBody>) -> <Self as Service>::Future {
        let mut response = Response::new(Body::empty());

        // TODO: Figure out how to extract the next two statements into a function Request<T> -> Vec<&str>
        let path: String = req.uri().path().to_owned();
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();

        debug!("Path segments: {:?}", segments);

        match (req.method(), &segments[..]) {
            (&Method::GET, []) => {
                return UserApiSession::build_response(Response::new(Body::from(HEALTHY_MESSAGE)));
            }
            (&Method::GET, ["topic", topic]) => return self.handle_topic_query(topic),
            (&Method::POST, ["topic", topic]) => return self.handle_topic_publish(topic, req),
            (&Method::GET, _) => {
                return UserApiSession::build_response(UserApiSession::not_found_response(
                    RESOURCE_NOT_FOUND_MESSAGE,
                ));
            }
            _ => *response.status_mut() = StatusCode::NOT_FOUND,
        }

        Box::new(future::ok(response))
    }
}

pub type ChunkStreamError = Box<(dyn std::error::Error + Sync + Send)>;
type ChunkStream = Box<dyn Stream<Item = Chunk, Error = ChunkStreamError> + Send + 'static>;

const HEALTHY_MESSAGE: &'static str = "{\"status\":\"HEALTHY\"}";
const RESOURCE_NOT_FOUND_MESSAGE: &'static str =
    "{\"error\":\"No resource found at the given URL\"}";
const TOPIC_NOT_FOUND_MESSAGE: &'static str = "{\"error\":\"topic not found\"}";
const INTERNAL_SERVER_ERROR_MESSAGE: &'static str =
    "{\"error\":\"internal server error occurred!\"}";

impl UserApiSession {
    fn handle_topic_query(&self, topic: &str) -> <Self as Service>::Future {
        debug!("Querying '{}' from connection {}", topic, self.remote);

        return Box::new(open_topic_log_read(topic).then(|result| match result {
            Ok(file) => {
                debug!("Streaming data to client");
                future::ok(Self::topic_response(file))
            }
            Err(UserApiError::LogOpeningError(error)) => {
                info!("Failed to open file. {}", error);
                future::ok(Self::not_found_response(TOPIC_NOT_FOUND_MESSAGE))
            }
            Err(other_error) => {
                error!("Failed to open file. {}", other_error);
                future::ok(Self::serverside_error_response())
            }
        }));
    }

    pub fn topic_response(file: File) -> Response<<Self as Service>::ResBody> {
        let chunk_source: ChunkStream = Box::new(
            create_chunk_source(file)
                .inspect_err(|e| error!("Failed to read file. {}", e))
                .map_err(|e| Box::new(e) as ChunkStreamError),
        );
        let body = Body::from(chunk_source);
        return Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .unwrap();
    }

    pub fn not_found_response(msg: &'static str) -> Response<<Self as Service>::ResBody> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(msg))
            .unwrap()
    }

    pub fn serverside_error_response() -> Response<<Self as Service>::ResBody> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(INTERNAL_SERVER_ERROR_MESSAGE))
            .unwrap()
    }

    pub fn build_response(
        response: Response<<Self as Service>::ResBody>,
    ) -> <Self as Service>::Future {
        Box::new(future::ok(response))
    }

    fn handle_topic_publish(
        &self,
        topic: &str,
        req: Request<<UserApiSession as Service>::ReqBody>,
    ) -> <Self as Service>::Future {
        debug!("Publishing to '{}' from connection {}", topic, self.remote);
        let chunk_source = req.into_body().map_err(UserApiError::BodyAccessError);
        let pipe = open_topic_log_overwrite(topic)
            .map(create_chunk_sink)
            .and_then(|chunk_sink| chunk_source.forward(chunk_sink));
        // Some coercion required to use the future result as a result body
        let response_future: ResponseStream = Box::new(
            pipe.into_stream()
                .inspect_err(|e| error!("Failed to process request. {}", e))
                .map(|_| Chunk::from("{}"))
                .map_err(|e| Box::new(e) as ResponseStreamError),
        );

        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(response_future))
            .unwrap();

        Box::new(future::ok(response))
    }
}

fn log_path(topic: &str) -> PathBuf {
    Path::new(LOG_DIR).join(Path::new(&(topic.to_owned() + ".log")))
}

fn open_topic_log_overwrite(topic: &str) -> impl Future<Item = File, Error = UserApiError> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .read(false)
        .truncate(false)
        .open(log_path(topic))
        .map_err(UserApiError::LogOpeningError)
}

fn create_chunk_sink(file: File) -> impl Sink<SinkItem = Chunk, SinkError = UserApiError> {
    tokio_codec::Decoder::framed(ChunksCodec::new(), file).sink_map_err(UserApiError::LogWriteError)
}

fn open_topic_log_read(topic: &str) -> impl Future<Item = File, Error = UserApiError> {
    OpenOptions::new()
        .write(false)
        .create(false)
        .read(true)
        .truncate(false)
        .open(log_path(topic))
        .map_err(UserApiError::LogOpeningError)
}

fn create_chunk_source(file: File) -> impl Stream<Item = Chunk, Error = UserApiError> {
    tokio_codec::Decoder::framed(ChunksCodec::new(), file).map_err(UserApiError::LogReadError)
}
