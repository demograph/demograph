use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;

use futures::future;
use futures::Sink;
use hyper::rt::{Future, Stream};
use hyper::service::Service;
use hyper::Chunk;
use hyper::{Body, Request, Response, StatusCode};
use tokio::fs::OpenOptions;
use tokio_fs::File;

use crate::api::http;
use crate::api::http::chunks_codec::ChunksCodec;
use crate::api::http::error::UserApiError;
use crate::api::http::ChunkStream;
use crate::api::http::ChunkStreamError;
use crate::LOG_DIR;

pub struct UserApiSession {
    remote: SocketAddr,
}

impl UserApiSession {
    pub fn new(remote: SocketAddr) -> UserApiSession {
        UserApiSession { remote }
    }

    /**
     *
     */
    pub fn handle_topic_query(&self, topic: &str) -> <Self as Service>::Future {
        debug!("Querying '{}' from connection {}", topic, self.remote);

        return Box::new(open_topic_log_read(topic).then(|result| match result {
            Ok(file) => {
                debug!("Streaming data to client");
                future::ok(Self::topic_response(file))
            }
            Err(UserApiError::LogOpeningError(error)) => {
                info!("Failed to open file. {}", error);
                future::ok(Self::not_found_response(http::TOPIC_NOT_FOUND_MESSAGE))
            }
            Err(other_error) => {
                error!("Failed to open file. {}", other_error);
                future::ok(Self::serverside_error_response())
            }
        }));
    }

    pub fn handle_topic_publish(
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
        let response_future: ChunkStream = Box::new(
            pipe.into_stream()
                .inspect_err(|e| error!("Failed to process request. {}", e))
                .map(|_| Chunk::from("{}"))
                .map_err(|e| Box::new(e) as ChunkStreamError),
        );

        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(response_future))
            .unwrap();

        Box::new(future::ok(response))
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
            .body(Body::from(http::INTERNAL_SERVER_ERROR_MESSAGE))
            .unwrap()
    }

    pub fn build_response(
        response: Response<<Self as Service>::ResBody>,
    ) -> <Self as Service>::Future {
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
