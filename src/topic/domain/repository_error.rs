quick_error! {
    #[derive(Debug)]
    pub enum TopicRepositoryError {
        TopicCreationFailed(id: &'static str, err: crate::topic::TopicCreationError)
        TopicRetrievalFailed(id: &'static str, err: crate::topic::TopicRetrievalError)
        TopicDeletionFailed(id: &'static str, err: crate::topic::TopicDeletionError)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TopicCreationError {
        IOFailed(err: std::io::Error)
        TopicAlreadyExists{}
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TopicRetrievalError {
        IOFailed(err: std::io::Error)
        NoSuchTopic{}
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TopicDeletionError {
        IOFailed(err: std::io::Error)
        NoSuchTopic{}
    }
}
