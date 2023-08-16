use time::UtcOffset;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt, fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

pub use tracing::{debug, error, info, warn};

pub fn init_logger() {
    let log_layer = fmt::layer()
        .with_timer(OffsetTime::new(
            UtcOffset::from_hms(8, 0, 0).unwrap(),
            &super::TIME_FORMAT,
        ))
        .with_file(true)
        .with_line_number(true)
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(log_layer).init();
}
