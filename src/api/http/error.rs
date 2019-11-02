use crate::repository::TopicRepositoryError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UserApiError {
    BindError(),
    ConnectionServeError(),
    BodyAccessError(hyper::Error),
    RequestJsonBodyParseError(serde_json::error::Error),
    TopicIOError(TopicRepositoryError),
    TopicJsonError(serde_json::error::Error),
    TopicWebsocketError(websock::Error),
}

impl fmt::Display for UserApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            UserApiError::BindError() => write!(f, "BindError"),
            UserApiError::ConnectionServeError() => write!(f, "ConnectionServeError"),
            UserApiError::BodyAccessError(x) => write!(f, "BodyAccessError({})", x),
            UserApiError::RequestJsonBodyParseError(x) => {
                write!(f, "RequestJsonBodyParseError({})", x)
            }
            UserApiError::TopicIOError(x) => write!(f, "TopicIOError({})", x),
            UserApiError::TopicJsonError(x) => write!(f, "TopicJsonError({})", x),
            UserApiError::TopicWebsocketError(x) => write!(f, "TopicWebsocketError({})", x),
        }
    }
}

impl Error for UserApiError {}

impl From<TopicRepositoryError> for UserApiError {
    fn from(error: TopicRepositoryError) -> Self {
        UserApiError::TopicIOError(error)
    }
}
