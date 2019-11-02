use std::net::SocketAddr;

use hyper::rt::Future;

use hyper::service::make_service_fn;
use hyper::Server;

use crate::api::http::session::UserApiSession;
use crate::repository::TopicRepository;

pub fn init_server<A: Into<SocketAddr>, TR: TopicRepository + Send + Sync + 'static>(
    address: A,
    topic_repository: TR,
) {
    let socket_address = address.into();
    //    let listener = TcpListener::bind2(&socket_address).expect("Failed to bind server to address");
    //    let http = Http::new();
    //    let server = listener
    //        .incoming()
    //        .for_each(move |(tcp_stream, addr)| {
    //            let session = UserApiSession::new(addr, topic_repository.clone());
    //            tokio::spawn(
    //                http.serve_connection(tcp_stream, session)
    //                    .map(|_| ())
    //                    .map_err(|_| ()),
    //            );
    //            Ok(())
    //        })
    //        .map_err(|e| panic!("accept error: {}", e));

    let make_service = make_service_fn(move |_| UserApiSession::new(topic_repository.clone()));

    let server = Server::bind(&socket_address)
        .serve(make_service)
        .map_err(|e| eprintln!("server error: {}", e));

    info!("Listening on http://{}", socket_address);
    hyper::rt::run(server);
}
