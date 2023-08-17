// use time::util::local_offset;
// use time::UtcOffset;
use tracing::{metadata::LevelFilter, Level};
use tracing_subscriber::{
    fmt::{self, time::OffsetTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

pub use tracing::{debug, error, info, warn};

use crate::infrastructure::config;

pub fn init_logger() {
    // unsafe {
    //     local_offset::set_soundness(local_offset::Soundness::Unsound);
    // }
    let log_level: Level;
    {
        let cc = config::C.lock().unwrap();
        let cc = cc.borrow();
        let log_conf = &cc.log;
        log_level = log_conf.level.parse().expect("wrong log level");
    }

    let log_layer = fmt::layer()
        // .with_timer(LocalTime::new(&config::TIME_FORMAT))
        .with_timer(OffsetTime::new(
            // UtcOffset::from_hms(8, 0, 0).unwrap(),
            *config::LOCAL_OFFSET,
            &config::TIME_FORMAT,
        ))
        .with_file(true)
        .with_line_number(true)
        .with_filter(LevelFilter::from_level(log_level));
    tracing_subscriber::registry().with(log_layer).init();
}
