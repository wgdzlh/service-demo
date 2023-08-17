use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(skip)]
    pub run_local: bool,
    pub port: Option<u16>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PgSqlConfig {
    pub url: String,
    #[serde(default)]
    pub log_mode: bool,
    #[serde(default)]
    pub auto_migrate: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub log: LogConfig,
    pub db: PgSqlConfig,
}

impl Config {
    pub fn parse(input: &str) -> Self {
        toml::from_str(input).unwrap()
    }
}
