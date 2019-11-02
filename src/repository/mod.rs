use futures::Future;

pub use error::*;
pub use plain_file_repository::*;

mod error;
pub mod flatten_sink;
mod plain_file_repository;

#[cfg(test)]
mod mod_test;

pub trait TopicRepository: Clone + Send + 'static {
    type Topic: crate::domain::TopicOld + Sized + Send + Sync + 'static;

    /** Obtain a Topic for the given name, creating it if it did not exist previously */
    fn load(
        &self,
        name: String,
    ) -> Box<dyn Future<Item = Self::Topic, Error = error::TopicRepositoryError> + Send>;

    /** Attempt to obtain a Topic for the given name, assuming that it exists */
    fn reload(
        &self,
        name: String,
    ) -> Box<dyn Future<Item = Self::Topic, Error = error::TopicRepositoryError> + Send>;

    fn remove(
        &self,
        name: String,
    ) -> Box<dyn Future<Item = (), Error = error::TopicRepositoryError> + Send>;
}
