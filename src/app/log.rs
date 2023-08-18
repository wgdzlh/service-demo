// use time::util::local_offset;
// use time::UtcOffset;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt::{self, time::OffsetTime},
    layer::SubscriberExt,
    reload,
    util::SubscriberInitExt,
    Layer,
};

pub use tracing::{debug, error, info, warn};

use crate::infrastructure::config;

pub fn init_logger() -> super::Result<()> {
    // unsafe {
    //     local_offset::set_soundness(local_offset::Soundness::Unsound);
    // }
    let log_conf = config::get_config()?.log;
    let log_level: LevelFilter = log_conf.level.parse().expect("wrong log level");

    let log_layer = fmt::layer()
        // .with_timer(LocalTime::new(&config::TIME_FORMAT))
        .with_timer(OffsetTime::new(
            // UtcOffset::from_hms(8, 0, 0)?,
            *config::LOCAL_OFFSET,
            &config::TIME_FORMAT,
        ))
        .with_file(true)
        .with_line_number(true)
        .with_filter(log_level);

    let (log_layer, reload_handle) = reload::Layer::new(log_layer);

    let reload_handle = Box::leak(Box::new(reload_handle)); // make handle static

    tracing_subscriber::registry().with(log_layer).init();

    config::add_callback(Box::new(|new, old| {
        if new.log.level != old.log.level {
            reload_handle
                .modify(|layer| {
                    if let Ok(level) = new.log.level.parse::<LevelFilter>() {
                        *layer.filter_mut() = level;
                    }
                })
                .ok();
        }
    }))?;
    Ok(())
}
