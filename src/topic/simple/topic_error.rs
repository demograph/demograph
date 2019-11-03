use crate::topic::TopicAccessError::IOFailed;
use crate::topic::TopicError;
use crate::topic::TopicError::TopicAccessFailed;
use futures::sync::mpsc::SendError as FutSendError;
use std::io::Error;
use tokio::io::ErrorKind;
use tokio::sync::mpsc::error::RecvError as TokioRcvError;
use tokio::sync::mpsc::error::SendError as TokioSendError;

impl From<TokioRcvError> for TopicError {
    fn from(err: TokioRcvError) -> Self {
        TopicAccessFailed(IOFailed(Error::new(ErrorKind::BrokenPipe, err)))
    }
}

impl From<TokioSendError> for TopicError {
    fn from(err: TokioSendError) -> Self {
        TopicAccessFailed(IOFailed(Error::new(ErrorKind::BrokenPipe, err)))
    }
}

impl<T: Send + Sync + 'static> From<FutSendError<T>> for TopicError {
    fn from(err: FutSendError<T>) -> Self {
        TopicAccessFailed(IOFailed(Error::new(ErrorKind::BrokenPipe, err)))
    }
}
