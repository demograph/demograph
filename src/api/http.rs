use crate::publisher::session::UserApiSession;
use hyper::rt::{Future, Stream};
use hyper::server::conn::Http;
use hyper::Chunk;
use std::net::SocketAddr;
use tokio_core::net::*;

pub type ResponseStreamError = Box<(dyn std::error::Error + Sync + Send + 'static)>;
pub type ResponseStream =
    Box<(dyn futures::Stream<Error = ResponseStreamError, Item = Chunk> + Send + 'static)>;

pub fn init_server<A: Into<SocketAddr>>(address: A) {
    let socket_address = address.into();
    let listener = TcpListener::bind2(&socket_address).expect("Failed to bind server to address");
    let http = Http::new();
    let server = listener
        .incoming()
        .for_each(move |(tcp_stream, addr)| {
            tokio::spawn(
                http.serve_connection(tcp_stream, UserApiSession::new(addr))
                    .map(|_| ())
                    .map_err(|_| ()),
            );
            Ok(())
        })
        .map_err(|e| panic!("accept error: {}", e));

    info!("Listening on http://{}", socket_address);
    hyper::rt::run(server);
}
