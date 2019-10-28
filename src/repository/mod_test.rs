mod tests {
    use std::io::Stderr;

    use futures::{future, Future, Sink, Stream};
    use hyper::Chunk;
    use serde_json::Value;

    use crate::domain::Topic;
    use crate::repository::TopicRepositoryError;

    #[test]
    fn future_compilation_test2() {
        let x: String = "abc".parse().unwrap();
        x.merge_patch(Value::Null);
    }

    impl Topic for String {
        fn chunk_sink(
            &self,
        ) -> Box<dyn Sink<SinkItem = Chunk, SinkError = TopicRepositoryError> + Send> {
            unimplemented!()
        }

        fn chunk_source(
            &self,
        ) -> Box<dyn Stream<Item = Chunk, Error = TopicRepositoryError> + Send> {
            unimplemented!()
        }

        fn read_as_json(
            &self,
        ) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send> {
            Box::new(future::ok(Value::Null))
        }

        fn write_as_json(
            &self,
            patch: Value,
        ) -> Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send> {
            Box::new(future::ok(()))
        }

        fn merge_patch(
            &self,
            patch: Value,
        ) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send> {
            let self_clone = self.clone();
            Box::new(
                self.read_as_json()
                    .and_then(move |x| self_clone.write_as_json(x).map(|_| Value::Null)),
            )
        }
    }

    // Not a problem
    #[test]
    fn future_compilation_test() {
        merge_patch(Value::Null);
    }

    fn read_as_json() -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send> {
        Box::new(future::ok(Value::Null))
    }

    fn write_as_json(v: Value) -> Box<dyn Future<Item = (), Error = TopicRepositoryError> + Send> {
        Box::new(future::ok(()))
    }

    fn merge_patch(
        patch: Value,
    ) -> Box<dyn Future<Item = Value, Error = TopicRepositoryError> + Send> {
        Box::new(read_as_json().and_then(|x| write_as_json(x).map(|_| Value::Null)))
    }
}
