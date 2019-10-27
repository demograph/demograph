#[macro_use]
extern crate log;

mod api;
mod domain;
mod repository;

use api::http::server::init_server;
use log::LevelFilter;
use repository::PlainFileRepository;
use simplelog::*;
use std::fs::create_dir;
use std::path::Path;

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
