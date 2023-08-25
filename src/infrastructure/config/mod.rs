mod conf;
mod nacos;

use std::{
    fs,
    path::Path,
    sync::{Mutex, MutexGuard},
};

use const_format::concatcp;
use futures::{channel::mpsc, SinkExt, StreamExt};
use notify::{
    event::{AccessKind, AccessMode},
    EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
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

const DEFAULT_LOG_LEVEL: &str = "info";
const LOCAL_CONF: &str = "./config.toml";

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
        let cs = fs::read_to_string(LOCAL_CONF)?;
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

fn set_config(mut new_conf: Config, init: bool) -> Result<()> {
    if new_conf.log.level.is_empty() {
        new_conf.log.level = DEFAULT_LOG_LEVEL.to_owned();
    }
    if init {
        *C.lock()? = new_conf;
    } else {
        let old_conf = C.lock()?.clone();
        *C.lock()? = new_conf.clone();
        let mut cbs = CALLBACKS.lock()?;
        for f in cbs.iter_mut() {
            f(&new_conf, &old_conf);
        }
    }
    Ok(())
}

// pub fn get_config() -> Result<Config> {
//     Ok(C.lock()?.clone())
// }

pub fn peek_config<'a>() -> Result<MutexGuard<'a, Config>> {
    Ok(C.lock()?)
}

pub fn is_local() -> bool {
    C.lock().map_or(false, |v| v.server.run_local)
}

pub fn add_callback(f: Callback) -> Result<()> {
    CALLBACKS.lock()?.push(f);
    Ok(())
}

pub async fn local_conf_watch() -> app::Result<()> {
    let (mut tx, mut rx) = mpsc::channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(tx.send(res)).ok();
        },
        notify::Config::default(),
    )?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new(LOCAL_CONF), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                debug!("changed: {:?}", event.kind);
                if event.kind != EventKind::Access(AccessKind::Close(AccessMode::Write)) {
                    continue;
                }
                if let Ok(cs) = fs::read_to_string(LOCAL_CONF) {
                    if let Ok(mut initial_conf) = Config::parse(&cs) {
                        initial_conf.server.run_local = true;
                        if set_config(initial_conf, false).is_ok() {
                            info!("local config updated");
                        }
                    }
                }
            }
            Err(e) => error!("watch error: {e:?}"),
        }
    }

    Ok(())
}
