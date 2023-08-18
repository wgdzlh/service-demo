#![warn(clippy::all)]
// #![allow(dead_code)]

mod app;
mod doc;
mod domain;
mod infrastructure;
mod interface;

pub use domain::{entity, repository};

use app::log::*;

use infrastructure::config;

fn main() -> app::Result<()> {
    config::init()?;
    init_logger()?;

    let version = option_env!("APP_VERSION").unwrap_or("dev");
    let build_time = option_env!("BUILD_TIME").unwrap_or("unknown");

    info!(?version, ?build_time, "service-demo starting...");
    app::cmd::run()
}
