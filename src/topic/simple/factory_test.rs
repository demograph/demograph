mod tests {
    use crate::topic::simple::fixtures::*;
    use crate::topic::simple::VolatileTopicFactory;
    use crate::topic::simple::*;
    use crate::topic::*;
    use futures::future::Future;
    use std::path::{Path, PathBuf};

    #[test]
    fn create__spawns_topic() {
        let factory = VolatileTopicFactory::<String>::new();
        let topic = factory.create(test_string()).wait();

        assert!(topic.is_ok());
    }
}
