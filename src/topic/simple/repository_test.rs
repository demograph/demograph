mod tests {
    use crate::topic::simple::fixtures::*;
    use crate::topic::simple::*;
    use crate::topic::*;
    use futures::future::Future;
    use std::io::Lines;
    use std::path::{Path, PathBuf};

    #[test]
    fn save__should__persist_current_state() {
        let repository = test_repository();
        let mut topic = test_string_topic();

        let maybe_topic = repository
            .save(test_string(), &mut topic, LinesCodec::new())
            .wait();

        assert!(maybe_topic.is_ok());
        assert!(topic_path(&test_string()).exists())
    }

    #[test]
    fn save__should__persist_updates() {}
}
