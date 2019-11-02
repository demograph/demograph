use std::error::Error;

use std::prelude::v1::Vec;

use futures::stream::Stream;
use futures::{future, IntoFuture};
use hyper::rt::Future;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, StatusCode};

use websock::Message;

use crate::api::http;
use crate::api::http::error::UserApiError;
use crate::api::http::error::UserApiError::{BodyAccessError, TopicWebsocketError};
use crate::api::http::session::UserApiSession;

use crate::repository::TopicRepository;

//impl<TR: TopicRepository + 'static> MakeService<TR> for UserApiSession<TR> {
//    type ReqBody = <UserApiSession<TR> as Service>::ReqBody;
//    type ResBody = <UserApiSession<TR> as Service>::ResBody;
//    type Error = <<UserApiSession<TR> as Service>::ReqBody as Trait>::Error;
//    type Service = UserApiSession<TR>;
//    type Future = Box<dyn Future<Item = Self::Service, Error = Self::MakeError>>;
//    type MakeError = <UserApiSession<TR> as Service>::Error;
//
//    fn make_service(&mut self, ctx: TR) -> Self::Future {
//        Box::new(future::ok(UserApiSession::new(ctx)))
//    }
//}

impl<TR: TopicRepository + Send + Sync + 'static> IntoFuture for UserApiSession<TR> {
    type Future = Box<dyn Future<Item = Self::Item, Error = Self::Error> + Send + Sync>;
    type Item = UserApiSession<TR>;
    type Error = Box<dyn Error + Send + Sync>;

    fn into_future(self) -> Self::Future {
        Box::new(future::ok(self))
    }
}

impl<TR: TopicRepository + Send + 'static> Service for UserApiSession<TR> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = UserApiError;
    type Future = Box<dyn Future<Item = Response<Body>, Error = UserApiError> + Send>;

    fn call(&mut self, req: Request<<Self as Service>::ReqBody>) -> <Self as Service>::Future {
        let mut response = Response::new(Body::empty());

        // TODO: Figure out how to extract the next two statements into a function Request<T> -> Vec<&str>
        let path: String = req.uri().path().to_owned();
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();

        debug!("Path segments: {:?}", segments);

        // Routing
        match (req.method(), &segments[..]) {
            (&Method::GET, []) => {
                return UserApiSession::<TR>::build_response(Response::new(Body::from(
                    http::HEALTHY_MESSAGE,
                )));
            }
            (&Method::GET, ["topic", topic]) => {
                if websock::is_websocket_upgrade(req.headers()) {
                    debug!("Spawning websocket");

                    let chunks = self
                        .handle_topic_query(topic)
                        .and_then(|response| {
                            response
                                .into_body()
                                .concat2()
                                .map(|chunk| chunk.to_vec())
                                .map_err(BodyAccessError)
                        })
                        .map_err(|_| websock::Error::InvalidMessageType);

                    let fws = chunks.map(|chunk| {
                        websock::spawn_websocket(req, move |m: Message<Vec<u8>>| {
                            debug!("Got message {:?}", m);
                            Box::new(future::ok(Some(websock::Message::binary(
                                // TODO: We really shouldn't be cloning here!!!
                                chunk.clone(),
                                m.context(),
                            ))))
                        })
                    });

                    //                    let ws = websock::spawn_websocket(req, move |m: Message<Vec<u8>>| {
                    //                        debug!("Got message {:?}", m);
                    //                        Box::new(
                    //                            chunks.map(|chunk| Some(websock::Message::binary(chunk, m.context()))),
                    //                        )
                    //                    });

                    //                    return Box::new(future::ok(ws));
                    return Box::new(fws.map_err(TopicWebsocketError));
                } else {
                    return self.handle_topic_query(topic);
                }
            }
            (&Method::POST, ["topic", topic]) => return self.handle_topic_publish(topic, req),
            (&Method::DELETE, ["topic", topic]) => return self.handle_topic_deletion(topic),
            (&Method::PATCH, ["topic", topic]) => return self.handle_topic_update(topic, req),
            (&Method::GET, _) => {
                return UserApiSession::<TR>::build_response(
                    UserApiSession::<TR>::not_found_response(http::RESOURCE_NOT_FOUND_MESSAGE),
                );
            }
            _ => *response.status_mut() = StatusCode::NOT_FOUND,
        }

        Box::new(future::ok(response))
    }
}
