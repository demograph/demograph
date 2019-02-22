use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UserApiError {
    BindError(),
    ConnectionServeError(),
    BodyAccessError(hyper::Error),
    LogOpeningError(std::io::Error),
    LogReadError(std::io::Error),
    LogWriteError(std::io::Error),
}

impl fmt::Display for UserApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            UserApiError::BindError() => write!(f, "BindError"),
            UserApiError::ConnectionServeError() => write!(f, "ConnectionServeError"),
            UserApiError::BodyAccessError(x) => write!(f, "BodyAccessError({})", x),
            UserApiError::LogOpeningError(x) => write!(f, "LogOpeningError({})", x),
            UserApiError::LogReadError(x) => write!(f, "LogReadError({})", x),
            UserApiError::LogWriteError(x) => write!(f, "LogWriteError({})", x),
        }
    }
}

impl Error for UserApiError {}
