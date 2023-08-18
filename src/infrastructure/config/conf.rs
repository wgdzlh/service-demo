use serde::{Deserialize, Serialize};

use crate::app;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(skip_deserializing)]
    pub run_local: bool,
    pub port: Option<u16>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PgSqlConfig {
    pub url: String,
    #[serde(default)]
    pub log_mode: bool,
    #[serde(default)]
    pub auto_migrate: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub log: LogConfig,
    pub db: PgSqlConfig,
}

impl Config {
    pub fn parse(input: &str) -> app::Result<Self> {
        Ok(toml::from_str(input)?)
    }
}
