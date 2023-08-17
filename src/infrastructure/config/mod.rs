use const_format::concatcp;
use once_cell::sync::Lazy;
use time::{
    format_description::{self, FormatItem},
    UtcOffset,
};

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
