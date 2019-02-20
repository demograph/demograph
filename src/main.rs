extern crate bytes;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_core;
extern crate tokio_fs;
extern crate tokio_io;
extern crate websocket;

use log::LevelFilter;
use simplelog::*;
use std::fs::create_dir;
use std::path::Path;

mod api;
mod chunks_codec;
mod publisher;
use api::http::init_server;

// TODO config
const LOG_DIR: &str = "./data/";

fn main() {
    init_logger();
    init_log_dir();
    // TODO: config
    init_server(([127, 0, 0, 1], 3000));
}

fn init_logger() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Debug, Config::default()).unwrap(),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            std::fs::File::create("app.log").unwrap(),
        ),
    ])
    .unwrap();
}

fn init_log_dir() {
    let path = Path::new(LOG_DIR);
    if !path.exists() && create_dir(path).is_err() {
        panic!("Failed to create log directory {:#?}", path);
    }
}
