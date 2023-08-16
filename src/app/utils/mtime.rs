use serde::{Deserialize, Deserializer, Serializer};

use crate::{app, entity::DateTimeTZ};

pub fn get_current_time() -> DateTimeTZ {
    DateTimeTZ::now_local().unwrap_or(DateTimeTZ::now_utc())
}

pub fn get_current_time_str() -> String {
    get_current_time()
        .format(&app::TIME_FORMAT)
        .unwrap_or_default()
}

// The signature of a serialize_with function must follow the pattern:
//
//    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer
//
// although it may also be generic over the input types T.
pub fn serialize<S>(date: &DateTimeTZ, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.format(&app::JSON_TIME_FORMAT).unwrap_or_default();
    serializer.serialize_str(&s)
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTimeTZ, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTimeTZ::parse(&s, &app::JSON_TIME_FORMAT).map_err(serde::de::Error::custom)
}
