pub mod cmd;
pub mod log;
pub mod utils;

pub use anyhow::Error;

pub use anyhow::Result;

use once_cell::sync::Lazy;
use time::format_description::{self, FormatItem};

pub static TIME_FORMAT: Lazy<Vec<FormatItem>> = Lazy::new(|| {
    format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]")
        .unwrap()
});

pub static JSON_TIME_FORMAT: Lazy<Vec<FormatItem>> = Lazy::new(|| {
    format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap()
});
