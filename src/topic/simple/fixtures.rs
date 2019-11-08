use crate::topic::simple::{InMemTopic, SimpleTopicRepository};
/// Test Fixtures
use crate::topic::Merge;
use crate::topic::MergeError;
use bytes::BufMut;
use bytes::BytesMut;
use futures::Future;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs::OpenOptions;
use std::panic::UnwindSafe;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use std::{io, panic, thread};
use tokio::codec::{Decoder, Encoder};
use tokio::runtime;
use tokio::runtime::Runtime;

const TEST_STRING: &str = "test-string";
pub fn test_string() -> String {
    String::from(TEST_STRING)
}

pub fn test_topic<T: Merge + Send + Clone>(state: T) -> InMemTopic<T> {
    InMemTopic::new(test_string(), state)
}
pub fn test_string_topic() -> InMemTopic<String> {
    InMemTopic::new(test_string(), test_string())
}

const TEST_DATA_DIR: &'static str = "./data";
pub fn test_directory() -> &'static Path {
    Path::new(TEST_DATA_DIR)
}

pub fn topic_path(id: &String) -> PathBuf {
    Path::new(test_directory()).join(Path::new(&(id.to_owned() + ".log")))
}

pub fn topic_file() -> PathBuf {
    topic_path(&test_string())
}

pub fn test_repository() -> SimpleTopicRepository<String> {
    SimpleTopicRepository::<String>::new(test_directory())
}

pub fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .collect()
}

pub fn given_a_topic_name<F>(test: F)
where
    F: FnOnce(&String, &PathBuf) -> () + UnwindSafe,
{
    let topic_name = random_string();
    let topic_path = topic_path(&topic_name);

    println!(
        "Using topic-name {} for test with backing file '{:?}'",
        topic_name, topic_path
    );

    test(&topic_name, &topic_path);
}

pub fn given_a_topic_file<F>(test: F)
where
    F: FnOnce(&String, &PathBuf) -> () + UnwindSafe,
{
    given_a_topic_name(|topic_name, topic_path| {
        // Make sure the file exists prior to test
        OpenOptions::new().create(true).open(topic_path);

        // Perform the test
        let result = panic::catch_unwind(|| test(topic_name, topic_path));

        // And let's not leave trash hanging around tests
        if topic_path.exists() {
            assert!(std::fs::remove_file(topic_path).is_ok());
        }

        result.unwrap();
    })
}

pub fn given_no_topic_file<F>(test: F)
where
    F: FnOnce(&String, &PathBuf) -> () + UnwindSafe,
{
    given_a_topic_name(|topic_name, topic_path| {
        // Make sure the file is removed prior to test
        if topic_path.exists() {
            assert!(std::fs::remove_file(topic_path).is_ok());
        }

        // Perform the test
        let result = panic::catch_unwind(|| test(topic_name, topic_path));

        // And let's not leave trash hanging around tests
        if topic_path.exists() {
            assert!(std::fs::remove_file(topic_path).is_ok());
        }

        result.unwrap();
    })
}

pub trait FutureRunner {
    fn run(self) -> Runtime;
}

impl<F> FutureRunner for F
where
    F: Future<Item = (), Error = ()> + Send + 'static,
{
    fn run(self) -> Runtime {
        let mut runtime: Runtime = Runtime::new().unwrap();
        runtime.spawn(self);
        runtime
    }
}

pub fn eventually_panic_free<F: Fn() -> S + UnwindSafe + Clone, S>(closure: F) -> S {
    fn internal<F: Fn() -> S + UnwindSafe + Clone, S>(closure: F, duration: Duration) -> S {
        match panic::catch_unwind(closure.clone()) {
            Ok(s) => s,
            Err(_) => {
                thread::sleep(duration);
                println!("Waiting for {:?}", duration);
                internal::<F, S>(closure, duration * 2)
            }
        }
    }
    internal(closure, Duration::from_millis(1))
}

//fn eventually_ok<F>(closure: F) -> S {
//    fn internal<F, S>(closure: F, duration: Duration) -> S {
//        panic::catch_unwind(closure).or_else(|| {
//            thread::sleep(duration);
//            internal(closure, duration * 2)
//        })
//    }
//    internal(closure, Duration::from_millis(1))
//}

impl Merge for String {
    type MergeError = ();
    fn merge(&self, patch: &Self) -> Result<Self, Self::MergeError> {
        Ok(format!("{}{}", self, patch))
    }
}

//impl Merge for Rc<String> {
//    fn merge(&self, patch: &Self) -> Result<Self, Self::MergeError> {
//        Ok(Rc::new(format!("{}{}", self, &patch)))
//    }
//}

impl Merge for Arc<String> {
    type MergeError = ();
    fn merge(&self, patch: &Self) -> Result<Self, Self::MergeError> {
        Ok(Arc::new(format!("{}{}", self, patch)))
    }
}

pub struct LinesCodec {
    // Stored index of the next index to examine for a `\n` character.
    // This is used to optimize searching.
    // For example, if `decode` was called with `abc`, it would hold `3`,
    // because that is the next index to examine.
    // The next time `decode` is called with `abcde\n`, the method will
    // only look at `de\n` before returning.
    next_index: usize,
}

impl LinesCodec {
    pub fn new() -> LinesCodec {
        LinesCodec { next_index: 0 }
    }
}

impl Decoder for LinesCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<String>, io::Error> {
        // Look for a byte with the value '\n' in buf. Start searching from the search start index.
        if let Some(newline_offset) = buf[self.next_index..].iter().position(|b| *b == b'\n') {
            // Found a '\n' in the string.

            // The index of the '\n' is at the sum of the start position + the offset found.
            let newline_index = newline_offset + self.next_index;

            // Split the buffer at the index of the '\n' + 1 to include the '\n'.
            // `split_to` returns a new buffer with the contents up to the index.
            // The buffer on which `split_to` is called will now start at this index.
            let line = buf.split_to(newline_index + 1);

            // Trim the `\n` from the buffer because it's part of the protocol,
            // not the data.
            let line = &line[..line.len() - 1];

            // Convert the bytes to a string and panic if the bytes are not valid utf-8.
            let line = std::str::from_utf8(&line).expect("invalid utf8 data");

            // Set the search start index back to 0.
            self.next_index = 0;

            // Return Ok(Some(...)) to signal that a full frame has been produced.
            Ok(Some(line.to_string()))
        } else {
            // '\n' not found in the string.

            // Tell the next call to start searching after the current length of the buffer
            // since all of it was scanned and no '\n' was found.
            self.next_index = buf.len();

            // Ok(None) signifies that more data is needed to produce a full frame.
            Ok(None)
        }
    }
}

impl Encoder for LinesCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, line: String, buf: &mut BytesMut) -> Result<(), io::Error> {
        // It's important to reserve the amount of space needed. The `bytes` API
        // does not grow the buffers implicitly.
        // Reserve the length of the string + 1 for the '\n'.
        buf.reserve(line.len() + 1);

        // String implements IntoBuf, a trait used by the `bytes` API to work with
        // types that can be expressed as a sequence of bytes.
        buf.put(line);

        // Put the '\n' in the buffer.
        buf.put_u8(b'\n');

        // Return ok to signal that no error occured.
        Ok(())
    }
}
