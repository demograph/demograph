quick_error! {
    #[derive(Debug)]
    pub enum TopicError {
        TopicAccessFailed(err: crate::topic::TopicAccessError)
        TopicPatchFailed(err: crate::topic::TopicPatchError)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TopicAccessError {
        IOFailed(err: std::io::Error)
        StateParsingFailed{}
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TopicPatchError {
        IOFailed{}
        StateParsingFailed{}
        PatchParsingFailed{}
        MergeFailed(err: MergeError)
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum MergeError {
        SchemaError{}
        CausalConsistencyViolation{}
    }
}
