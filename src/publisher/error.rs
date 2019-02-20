use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PublisherError {
    BindError(),
    ConnectionServeError(),
    BodyAccessError(hyper::Error),
    LogOpeningError(std::io::Error),
    LogWriteError(std::io::Error),
}

impl fmt::Display for PublisherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            PublisherError::BindError() => write!(f, "BindError"),
            PublisherError::ConnectionServeError() => write!(f, "ConnectionServeError"),
            PublisherError::BodyAccessError(x) => write!(f, "BodyAccessError({})", x),
            PublisherError::LogOpeningError(x) => write!(f, "LogOpeningError({})", x),
            PublisherError::LogWriteError(x) => write!(f, "LogWriteError({})", x),
        }
    }
}

impl Error for PublisherError {}
