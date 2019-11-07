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
    fn save__should__persist_current_state() {
        let repository = test_repository();
        let mut topic = test_string_topic();

        let save_future = repository
            .save(test_string(), &mut topic, LinesCodec::new())
            .map_err(|_| ())
            .run();

        eventually_panic_free(|| {
            assert!(topic_path(&test_string()).exists());
        });
    }

    #[test]
    fn save__should__persist_updates() {}
}
