mod conf;
mod nacos;

use std::{cell::RefCell, fs, sync::Mutex};

use const_format::concatcp;
use once_cell::sync::Lazy;
use time::{
    format_description::{self, FormatItem},
    UtcOffset,
};

use conf::Config;

use crate::app;

pub const APP: &str = "service-demo";
pub const BASE_PATH: &str = concatcp!("/", APP, "/v1");

pub static C: Lazy<Mutex<RefCell<Config>>> = Lazy::new(Mutex::default);

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

pub fn init() -> app::Result<()> {
    let run_local = true;
    let mut ic;
    if run_local {
        let contents =
            fs::read_to_string("./config.toml").expect("config file ./config.toml not exist");
        ic = Config::parse(&contents);
        ic.server.run_local = true;
    } else {
        let cs = nacos::setup_nacos_conf_sub()?;
        ic = Config::parse(&cs);
    }
    println!("initial config: {}", serde_json::to_value(&ic)?);
    *C.lock().unwrap().borrow_mut() = ic;
    Ok(())
}
