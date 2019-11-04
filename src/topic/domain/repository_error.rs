quick_error! {
    #[derive(Debug)]
    pub enum TopicRepositoryError {
        TopicSaveFailed(id: &'static str, err: crate::topic::TopicSaveError)
        TopicRetrievalFailed(id: &'static str, err: crate::topic::TopicRetrievalError)
        TopicDeletionFailed(id: &'static str, err: crate::topic::TopicDeletionError)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TopicSaveError {
        IOFailed(err: std::io::Error)
        TopicFailed(err: crate::topic::TopicError)
//        TopicAlreadyExists{}
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
