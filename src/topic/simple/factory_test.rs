mod tests {
    use crate::topic::simple::SimpleTopicFactory;
    use crate::topic::simple::*;
    use crate::topic::*;
    use futures::future::Future;
    use std::path::{Path, PathBuf};

    #[test]
    fn create__spawns_topic() {
        let factory = SimpleTopicFactory::<String>::new();
        let topic = factory.create(String::from("initial-state")).wait();

        assert!(topic.is_ok());
    }
}
