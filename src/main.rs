#[macro_use]
extern crate log;

use std::fs::create_dir;
use std::path::Path;

use log::LevelFilter;
use simplelog::*;

use api::http::server::init_server;
use repository::PlainFileRepository;

mod api;
mod domain;
mod repository;

// TODO config
const LOG_DIR: &str = "./data/";

fn main() {
    init_logger();
    init_log_dir();
    // TODO: config
    init_server(([127, 0, 0, 1], 3000), PlainFileRepository::new(LOG_DIR));
}

fn init_logger() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Stdout).unwrap(),
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
