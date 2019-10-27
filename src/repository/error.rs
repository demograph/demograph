use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum TopicRepositoryError {
    TopicRemovalError(std::io::Error),
    TopicLoadError(std::io::Error),
    TopicReadError(std::io::Error),
    TopicWriteError(std::io::Error),
}

impl fmt::Display for TopicRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            TopicRepositoryError::TopicRemovalError(x) => write!(f, "TopicRemovalError({})", x),
            TopicRepositoryError::TopicLoadError(x) => write!(f, "TopicLoadError({})", x),
            TopicRepositoryError::TopicReadError(x) => write!(f, "TopicReadError({})", x),
            TopicRepositoryError::TopicWriteError(x) => write!(f, "TopicWriteError({})", x),
        }
    }
}

impl Error for TopicRepositoryError {}
