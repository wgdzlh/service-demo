use serde::{Deserialize, Deserializer, Serializer};

use crate::{entity::DateTimeTZ, infrastructure::config};

pub fn get_current_time() -> DateTimeTZ {
    // DateTimeTZ::now_local().unwrap_or_else(|_| DateTimeTZ::now_utc())
    DateTimeTZ::now_utc().to_offset(*config::LOCAL_OFFSET) // no need to detect offset every time
}

pub fn get_current_time_str() -> String {
    get_current_time()
        .format(&config::TIME_FORMAT)
        .unwrap_or_default()
}

/// JSON timestamp output format
pub fn serialize<S>(date: &DateTimeTZ, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date
        .to_offset(*config::LOCAL_OFFSET)
        .format(&config::JSON_TIME_FORMAT)
        .unwrap_or_default();
    serializer.serialize_str(&s)
}

/// JSON timestamp input parse
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTimeTZ, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(DateTimeTZ::parse(&s, &config::JSON_TIME_FORMAT)
        .map_err(serde::de::Error::custom)?
        .replace_offset(*config::LOCAL_OFFSET))
}
