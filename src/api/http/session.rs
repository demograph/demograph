use std::net::SocketAddr;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::Service;
use hyper::Chunk;
use hyper::{Body, Request, Response, StatusCode};

use crate::api::http;
use crate::api::http::error::UserApiError;
use crate::api::http::ChunkStream;
use crate::api::http::ChunkStreamError;
use crate::domain::Topic;
use crate::repository::{TopicRepository, TopicRepositoryError};

pub struct UserApiSession<TR: TopicRepository> {
    remote: SocketAddr,
    topic_repository: TR,
}

impl<TR: TopicRepository> UserApiSession<TR> {
    pub fn new(remote: SocketAddr, topic_repository: TR) -> UserApiSession<TR> {
        UserApiSession {
            remote,
            topic_repository,
        }
    }

    /**
     *
     */
    pub fn handle_topic_query(&self, topic: &str) -> <Self as Service>::Future {
        debug!("Querying '{}' from connection {}", topic, self.remote);

        let ftopic = self
            .topic_repository
            .reload(topic.to_owned())
            .from_err::<UserApiError>();

        return Box::new(ftopic.then(|result| match result {
            Ok(topic) => {
                debug!("Streaming data to client");
                future::ok(Self::topic_response(&topic))
            }
            Err(UserApiError::TopicIOError(TopicRepositoryError::TopicLoadError(error))) => {
                info!("Failed to load topic. {}", error);
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
        topic_name: &str,
        req: Request<<UserApiSession<TR> as Service>::ReqBody>,
    ) -> <Self as Service>::Future {
        debug!(
            "Publishing to '{}' from connection {}",
            topic_name, self.remote
        );
        let chunk_source = req.into_body().map_err(UserApiError::BodyAccessError);
        let pipe = self
            .topic_repository
            .load(topic_name.to_owned())
            .from_err::<UserApiError>()
            .map(|topic| topic.chunk_sink())
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

    pub fn topic_response(topic: &dyn Topic) -> Response<<Self as Service>::ResBody> {
        let chunk_source: ChunkStream = Box::new(
            topic
                .chunk_source()
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
