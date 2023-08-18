mod conf;
mod nacos;

use std::{fs, sync::RwLock};

use const_format::concatcp;
use once_cell::sync::Lazy;
use time::{
    format_description::{self, FormatItem},
    UtcOffset,
};

use conf::Config;

use crate::{app, repository::Result};

pub const APP: &str = "service-demo";
pub const BASE_PATH: &str = concatcp!("/", APP, "/v1");

pub static TIME_FORMAT: Lazy<Vec<FormatItem>> = Lazy::new(|| {
    format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]")
        .unwrap()
});

pub static JSON_TIME_FORMAT: Lazy<Vec<FormatItem>> = Lazy::new(|| {
    format_description::parse(
        "[year]-[month]-[day] [hour]:[minute]:[second]", // [offset_hour sign:mandatory]
    )
    .unwrap()
});

pub static LOCAL_OFFSET: Lazy<UtcOffset> = Lazy::new(|| UtcOffset::current_local_offset().unwrap());

static C: Lazy<RwLock<Config>> = Lazy::new(RwLock::default);

pub fn init() -> app::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    // println!("args: {args:?}");
    let run_local = matches!(&args[..], [_, v, ..] if v == "-l");
    let mut initial_conf;
    if run_local {
        let cs = fs::read_to_string("./config.toml").expect("local ./config.toml not exist");
        initial_conf = Config::parse(&cs)?;
        initial_conf.server.run_local = true;
    } else {
        let cs = nacos::setup_nacos_conf_sub()?;
        initial_conf = Config::parse(&cs)?;
    }
    println!("initial config: {}", serde_json::to_value(&initial_conf)?);
    set_config(initial_conf)?;
    Ok(())
}

fn set_config(new_conf: Config) -> Result<()> {
    *C.write()? = new_conf;
    Ok(())
}

pub fn get_config() -> Result<Config> {
    Ok(C.read()?.clone())
}
