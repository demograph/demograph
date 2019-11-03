mod tests {
    use crate::topic::simple::SimpleTopicFactory;
    use crate::topic::simple::*;
    use crate::topic::*;
    use futures::future::Future;
    use futures::stream::Stream;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::Arc;

    const TEST_STRING: &str = "test-string";
    fn test_string() -> String {
        String::from(TEST_STRING)
    }
    fn test_topic<T: Merge + Clone>(state: T) -> SimpleTopic<T> {
        SimpleTopic::new(test_string(), state)
    }

    #[test]
    fn snapshot__returns_initial_state() {
        let topic = test_topic(test_string());
        let snapshot = topic.snapshot().wait();

        assert!(snapshot.is_ok());
        assert_eq!(snapshot.unwrap(), test_string());
    }

    pub struct RefTest<'a> {
        x: String,
        y: &'a String,
    }

    #[test]
    fn snapshot__supports_rc() {
        let initial_state = Rc::new(test_string());
        let expected_state = initial_state.clone();

        let topic = test_topic(initial_state);
        let snapshot = topic.snapshot().wait();

        assert!(snapshot.is_ok());
        assert_eq!(snapshot.unwrap(), expected_state);
    }

    #[test]
    fn snapshot__supports_arc() {
        let initial_state = Arc::new(test_string());
        let expected_state = initial_state.clone();

        let topic = test_topic(initial_state);
        let snapshot = topic.snapshot().wait();

        assert!(snapshot.is_ok());
        assert_eq!(snapshot.unwrap(), expected_state);
    }

    #[test]
    fn patch_succeeds() {
        let mut topic = test_topic(test_string());
        let patched = topic.patch(test_string()).wait();

        assert!(patched.is_ok());
    }

    #[test]
    fn patch_updates_state() {
        let mut topic = test_topic(test_string());
        let updated_topic = topic.patch(test_string()).wait();

        assert!(updated_topic.is_ok());
        let updated_topic = updated_topic.unwrap();

        let snapshot = updated_topic.snapshot().wait();
        assert!(snapshot.is_ok());
        assert_eq!(
            snapshot.unwrap(),
            format!("{}{}", test_string(), test_string())
        );
    }

    #[test]
    fn subscribe__provides_initial_state() {
        let mut topic = test_topic(test_string());
        let result = topic.subscribe().into_future().wait();

        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap().0, Some(test_string()));
    }

    #[test]
    fn subscribe__provides_updates() {
        let mut topic = test_topic(test_string());

        let result = topic.subscribe().into_future().wait();
        assert!(result.is_ok());
        let head_stream = result.ok().unwrap();
        assert_eq!(head_stream.0, Some(test_string()));

        let patch = String::from("patching");
        // waiting necessary for stream to pick up element
        // without wait, the stream _does_ receive, but it's element is None
        let patched_topic = topic.patch(patch.clone()).wait();

        let result = head_stream.1.into_future().wait();
        assert!(result.is_ok());
        let head_stream = result.ok().unwrap();
        assert_eq!(head_stream.0, Some(patch));
    }

    impl Merge for String {
        fn merge(&self, patch: &Self) -> Result<Self, MergeError> {
            Ok(format!("{}{}", self, patch))
        }
    }

    impl Merge for Rc<String> {
        fn merge(&self, patch: &Self) -> Result<Self, MergeError> {
            Ok(Rc::new(format!("{}{}", self, &patch)))
        }
    }

    impl Merge for Arc<String> {
        fn merge(&self, patch: &Self) -> Result<Self, MergeError> {
            Ok(Arc::new(format!("{}{}", self, patch)))
        }
    }
}
