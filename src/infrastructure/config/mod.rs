mod conf;
mod nacos;

use std::{fs, sync::Mutex};

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
    println!(
        "initial config:\n{}",
        serde_json::to_string_pretty(&initial_conf)?
    );
    set_config(initial_conf, true)?;
    Ok(())
}

type Callback = Box<dyn FnMut(&Config, &Config) + Send + Sync>; // callback type for online config change; may mutate captured vars

static C: Lazy<Mutex<Config>> = Lazy::new(Mutex::default); // global config instance
static CALLBACKS: Lazy<Mutex<Vec<Callback>>> = Lazy::new(Mutex::default);

fn set_config(new_conf: Config, init: bool) -> Result<()> {
    if init {
        *C.lock()? = new_conf;
    } else {
        let old_conf = get_config()?;
        *C.lock()? = new_conf.clone();
        let mut cbs = CALLBACKS.lock()?;
        for f in cbs.iter_mut() {
            f(&new_conf, &old_conf);
        }
    }
    Ok(())
}

pub fn get_config() -> Result<Config> {
    Ok(C.lock()?.clone())
}

pub fn add_callback(f: Callback) -> Result<()> {
    CALLBACKS.lock()?.push(f);
    Ok(())
}
