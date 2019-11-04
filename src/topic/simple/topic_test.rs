mod tests {
    use crate::topic::simple::fixtures::*;
    use crate::topic::simple::VolatileTopicFactory;
    use crate::topic::simple::*;
    use crate::topic::*;
    use futures::future::Future;
    use futures::stream::Stream;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn snapshot__returns_initial_state() {
        let topic = test_string_topic();
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
        let mut topic = test_string_topic();
        let patched = topic.patch(test_string()).wait();

        assert!(patched.is_ok());
    }

    #[test]
    fn patch_updates_state() {
        let mut topic = test_string_topic();
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
        let mut topic = test_string_topic();
        let result = topic.subscribe().into_future().wait();

        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap().0, Some(test_string()));
    }

    #[test]
    fn subscribe__provides_updates() {
        let mut topic = test_string_topic();

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
}
