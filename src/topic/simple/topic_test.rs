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
        topic.patch(test_string()).wait();

        let snapshot = topic.snapshot().wait();
        assert!(snapshot.is_ok());
        assert_eq!(
            snapshot.unwrap(),
            format!("{}{}", test_string(), test_string())
        );
    }

    #[test]
    fn merge_updates_state__and__returns_result() {
        let mut topic = test_topic(test_string());
        let merge_result = topic.merge(test_string()).wait();
        assert!(merge_result.is_ok());
        assert_eq!(
            merge_result.unwrap(),
            format!("{}{}", test_string(), test_string())
        );

        let snapshot = topic.snapshot().wait();
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

    //    #[test] // does not support references in fact
    //    fn snapshot__supports_references() {
    //        let s = test_string();
    //        let initial_state = RefTest {
    //            x: test_string(),
    //            y: &s,
    //        };
    //
    //        let topic = SimpleTopic::<RefTest>::new(String::from(""), initial_state);
    //        let snapshot = topic.snapshot().wait();
    //
    //        assert!(snapshot.is_ok());
    //        assert_eq!(snapshot.unwrap().x, s);
    //        assert_eq!(snapshot.unwrap().y, s);
    //    }

    impl Merge for String {
        fn merge(&self, patch: Self) -> Result<Self, MergeError> {
            Ok(format!("{}{}", self, patch))
        }
    }

    impl Merge for Rc<String> {
        fn merge(&self, patch: Self) -> Result<Self, MergeError> {
            Ok(Rc::new(format!("{}{}", self, patch)))
        }
    }

    impl Merge for Arc<String> {
        fn merge(&self, patch: Self) -> Result<Self, MergeError> {
            Ok(Arc::new(format!("{}{}", self, patch)))
        }
    }
}
