use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum TopicStorageError {
    TopicOpeningError(std::io::Error),
    TopicReadError(std::io::Error),
    TopicWriteError(std::io::Error),
}

impl fmt::Display for TopicStorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            TopicStorageError::TopicOpeningError(x) => write!(f, "TopicOpeningError({})", x),
            TopicStorageError::TopicReadError(x) => write!(f, "TopicReadError({})", x),
            TopicStorageError::TopicWriteError(x) => write!(f, "TopicWriteError({})", x),
        }
    }
}

impl Error for TopicStorageError {}
