use hyper::Chunk;

mod chunks_codec;
mod error;
pub mod server;
mod session;
mod user_api_service;

pub type ChunkStreamError = Box<(dyn std::error::Error + Sync + Send)>;
pub type ChunkStream =
    Box<dyn futures::Stream<Item = Chunk, Error = ChunkStreamError> + Send + 'static>;

const HEALTHY_MESSAGE: &'static str = "{\"status\":\"HEALTHY\"}";
const RESOURCE_NOT_FOUND_MESSAGE: &'static str =
    "{\"error\":\"No resource found at the given URL\"}";
const TOPIC_NOT_FOUND_MESSAGE: &'static str = "{\"error\":\"topic not found\"}";
const INTERNAL_SERVER_ERROR_MESSAGE: &'static str =
    "{\"error\":\"internal server error occurred!\"}";
