use futures::future;

use hyper::rt::{Future, Stream};
use hyper::service::Service;
use hyper::Chunk;
use hyper::{Body, Request, Response, StatusCode};

use serde_json::Value;

use crate::api::http;

use crate::api::http::error::UserApiError;
use crate::api::http::error::UserApiError::{TopicIOError, TopicJsonError};

use crate::api::http::ChunkStream;
use crate::api::http::ChunkStreamError;
use crate::domain::TopicOld;
use crate::repository::{TopicRepository, TopicRepositoryError};

pub struct UserApiSession<TR: TopicRepository + Send> {
    //    remote: SocketAddr,
    topic_repository: TR,
}

impl<TR: TopicRepository + Send> UserApiSession<TR> {
    pub fn new(topic_repository: TR) -> UserApiSession<TR> {
        UserApiSession {
            //            remote,
            topic_repository,
        }
    }

    /**
     *
     */
    pub fn handle_topic_query(&self, topic: &str) -> <Self as Service>::Future {
        debug!(
            "Querying '{}' from connection {}",
            topic, "'no request access'"
        );

        let maybe_body = self
            .topic_repository
            .reload(topic.to_owned())
            .from_err::<UserApiError>()
            .map(|topic| {
                let chunk_source: ChunkStream = Box::new(
                    topic
                        .chunk_source()
                        .inspect_err(|e| error!("Failed to read file. {}", e))
                        .map_err(|e| Box::new(e) as ChunkStreamError),
                );

                Body::from(chunk_source)
            });

        Self::create_response(maybe_body)
    }

    //    pub fn handle_topic_websocket(
    //        &self,
    //        topic_name: &str,
    //    ) -> Box<dyn Future<Item = Option<Message<Value>>, Error = websock::Error> + Send> {
    //        let ftopic = self
    //            .topic_repository
    //            .reload(topic_name.to_owned())
    //            .from_err::<UserApiError>();
    //
    //        ftopic.map(|topic| topic.chunk_source())
    //    }

    pub fn handle_topic_deletion(&self, topic_name: &str) -> <Self as Service>::Future {
        debug!("Removing topic '{}' from connection", topic_name);
        let maybe_body = self
            .topic_repository
            .remove(topic_name.to_owned())
            .map_err(UserApiError::TopicIOError)
            .map(|_| Body::empty());

        Self::create_response(maybe_body)
    }

    pub fn handle_topic_update(
        &self,
        topic_name: &str,
        req: Request<<UserApiSession<TR> as Service>::ReqBody>,
    ) -> <Self as Service>::Future {
        debug!("Patching topic '{}' from connection", topic_name);

        let ftopic = self
            .topic_repository
            .reload(topic_name.to_owned())
            .from_err::<UserApiError>();

        let validated_patch = req
            .into_body()
            .map_err(UserApiError::BodyAccessError)
            .concat2()
            .and_then(move |body: Chunk| {
                serde_json::from_slice::<Value>(&body)
                    .map_err(UserApiError::RequestJsonBodyParseError)
            });

        let maybe_response = ftopic.join(validated_patch).map(|(topic, patch)| {
            let chunks = topic
                .merge_patch(patch)
                .inspect(|_result| debug!("post patch"))
                .map_err(TopicIOError)
                .and_then(|json| serde_json::to_vec(&json).map_err(TopicJsonError))
                .map(Chunk::from)
                .into_stream()
                .map_err(|e| Box::new(e) as ChunkStreamError);

            let stream: ChunkStream = Box::new(chunks);
            Body::from(stream)
        });

        Self::create_response(maybe_response)
    }

    pub fn handle_topic_publish(
        &self,
        topic_name: &str,
        req: Request<<UserApiSession<TR> as Service>::ReqBody>,
    ) -> <Self as Service>::Future {
        debug!("Publishing to '{}' from connection", topic_name);
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

    pub fn create_response<T>(maybe_body: T) -> <Self as Service>::Future
    where
        T: Future<Item = Body, Error = UserApiError> + Send + 'static,
    {
        Box::new(maybe_body.then(move |result| match result {
            Ok(body) => {
                debug!("Streaming data to client");

                let response = Response::builder()
                    .status(StatusCode::OK)
                    .body(body)
                    .unwrap();

                future::ok(response)
            }
            Err(UserApiError::TopicIOError(TopicRepositoryError::TopicLoadError(error))) => {
                info!("Failed to load topic. {}", error);
                future::ok(Self::not_found_response(http::TOPIC_NOT_FOUND_MESSAGE))
            }
            Err(UserApiError::TopicIOError(TopicRepositoryError::TopicRemovalError(error))) => {
                info!("Failed to remove topic. {}", error);
                future::ok(Self::not_found_response(http::TOPIC_NOT_FOUND_MESSAGE))
            }
            Err(other_error) => {
                error!("Failed to open file. {}", other_error);
                future::ok(Self::serverside_error_response())
            }
        }))
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
