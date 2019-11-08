mod tests {
    use crate::topic::simple::fixtures::*;
    use crate::topic::simple::*;
    use crate::topic::*;
    use futures::future::lazy;
    use futures::future::Future;
    use futures::{future, Async};
    use std::convert::TryInto;
    use std::io::Lines;
    use std::panic::UnwindSafe;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::{mem, panic, thread};
    use tokio;
    use tokio::runtime;
    use tokio::runtime::current_thread;
    use tokio::runtime::Runtime;

    #[test]
    fn save_should_persist_current_state() {
        given_no_topic_file(|topic_name, path| {
            // Given a topic
            let mut topic = test_string_topic();

            // When we save the topic to the repository
            let save_future = test_repository()
                .save(topic_name, &mut topic, LinesCodec::new())
                .map_err(|_| ())
                .run();

            // Then the topic file should eventually be created
            eventually_panic_free(|| {
                assert!(path.exists());
                let content = std::fs::read_to_string(path).unwrap();
                assert_eq!(content, format!("{}\n", test_string()))
            });
        })
    }

    #[test]
    fn save_should_persist_updates() {
        given_no_topic_file(|topic_name, path| {
            // Given a topic
            let mut topic = test_string_topic();

            // When we save the topic to the repository
            let save_future = test_repository()
                .save(topic_name, &mut topic, LinesCodec::new())
                .map_err(|_| ())
                .run();

            // And a write is made to the topic
            let another_string = random_string();
            topic.patch(another_string.clone()).wait();

            // Then the topic file should eventually contain the topic content
            eventually_panic_free(|| {
                let content = std::fs::read_to_string(path).unwrap();
                assert_eq!(content, format!("{}\n{}\n", test_string(), another_string))
            });
        });
    }

    #[test]
    fn delete_topic_should_delete_file() {
        given_a_topic_file(|topic_name, path| {
            // Given a topic
            let mut topic = test_string_topic();

            // When we delete the topic
            test_repository().delete(topic_name, topic);

            // Then the topic file should eventually be removed
            eventually_panic_free(|| assert!(!path.exists()));
        })
    }
}
