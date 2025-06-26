use super::DatabaseType;
use super::Redis;
use super::Secure;

use serde::{Deserialize, Serialize};

/// Config of madoka_auth.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Config file version.
    pub version: u8,
    /// Server hostname.
    /// Default hostname is "127.0.0.1".
    pub host: Option<String>,
    /// Server port.
    /// Default port is 7817.
    pub port: Option<u32>,
    /// Database type.
    /// Default type is "sqlite".
    pub database: Option<DatabaseType>,
    /// Redis configuration.
    pub redis: Redis,
    /// Secure configuration.
    pub secure: Option<Secure>,
}
