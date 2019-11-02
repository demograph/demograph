use crate::repository::TopicRepositoryError;
use crate::topic::simple::SimpleTopic;
use crate::topic::{Merge, Topic, TopicRepository};
use futures::{future, Future};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub struct SimpleTopicRepository<TS> {
    base_dir: &'static Path,
    _phantom: std::marker::PhantomData<TS>,
}

impl<TS> SimpleTopicRepository<TS> {
    pub fn new(base_dir: &'static Path) -> SimpleTopicRepository<TS> {
        SimpleTopicRepository {
            base_dir,
            _phantom: PhantomData,
        }
    }

    pub fn topic_path(&self, id: &String) -> PathBuf {
        Path::new(&self.base_dir).join(Path::new(&(id.to_owned() + ".log")))
    }
}

impl<TS: Merge + Clone + 'static> TopicRepository<TS> for SimpleTopicRepository<TS> {
    type TTS = SimpleTopic<TS>;
    type TopicFuture = Box<dyn Future<Item = Self::TTS, Error = TopicRepositoryError>>;
    type DeleteFuture = Box<dyn Future<Item = (), Error = TopicRepositoryError>>;

    fn save(&self, id: String, topic: Self::TTS) -> Self::TopicFuture {
        unimplemented!()
    }

    fn delete(&self, id: String, topic: Self::TTS) -> Self::DeleteFuture {
        unimplemented!()
    }

    fn load(&self, id: String) -> Self::TopicFuture {
        unimplemented!()
    }
}
