use bytes::{BufMut, BytesMut};
use hyper::Chunk;

use tokio::codec::Decoder;
use tokio::codec::Encoder;

pub struct ChunksCodec(());

impl ChunksCodec {
    pub fn new() -> ChunksCodec {
        ChunksCodec(())
    }
}

impl Decoder for ChunksCodec {
    type Item = Chunk;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Chunk>, std::io::Error> {
        let len = src.len();
        if len > 0 {
            Ok(Some(Chunk::from(src.split_to(len).to_vec())))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ChunksCodec {
    type Item = Chunk;
    type Error = std::io::Error;

    fn encode(&mut self, item: Chunk, dst: &mut BytesMut) -> Result<(), std::io::Error> {
        dst.reserve(item.len());
        dst.put(item);
        Ok(())
    }
}
