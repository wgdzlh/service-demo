mod conf;
mod nacos;

use std::{fs, path::Path, sync::Mutex};

use const_format::concatcp;
use notify::{RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use time::{
    format_description::{self, FormatItem},
    UtcOffset,
};

use conf::Config;

use crate::{
    app::{self, log::*},
    repository::Result,
};

pub const APP: &str = "service-demo";
pub const BASE_PATH: &str = concatcp!("/", APP, "/v1");
pub const DEFAULT_PORT: u16 = 8080;

pub const DEFAULT_LOG_LEVEL: &str = "info";

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
    println!("current timezone: {:?}", *LOCAL_OFFSET);
    let args: Vec<String> = std::env::args().collect();
    // println!("args: {args:?}");
    let run_local = matches!(&args[..], [_, v, ..] if v == "-l");
    let mut initial_conf;
    if run_local {
        let cs = fs::read_to_string("./config.toml").expect("local ./config.toml not exist");
        initial_conf = Config::parse(&cs)?;
        initial_conf.server.run_local = true;
        local_conf_watch()?;
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

fn set_config(mut new_conf: Config, init: bool) -> Result<()> {
    if new_conf.log.level.is_empty() {
        new_conf.log.level = DEFAULT_LOG_LEVEL.to_owned();
    }
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

fn local_conf_watch() -> app::Result<()> {
    // Automatically select the best implementation for your platform.
    let mut watcher = notify::recommended_watcher(|res| {
        match res {
            Ok(event) => info!("event: {:?}", event),
            Err(e) => error!("watch error: {:?}", e),
            // _ => {}
        }
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new("./config.toml"), RecursiveMode::NonRecursive)?;
    Ok(())
}
