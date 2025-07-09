use crate::internal::config::common::CONFIG_VERSION;

use super::Database;
use super::Mail;
use super::Redis;
use super::Secure;

use serde::{Deserialize, Serialize};

/// Config of madoka_auth.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Config file version.
    #[serde(default = "default_version")]
    pub version: u8,
    /// Server hostname.
    /// Default hostname is "127.0.0.1".
    #[serde(default = "default_host")]
    pub host: String,
    /// Server port.
    /// Default port is 7817.
    #[serde(default = "default_port")]
    pub port: u32,
    /// Database type.
    /// Default type is "sqlite".
    #[serde(default = "Default::default")]
    pub database: Database,
    /// Redis configuration.
    #[serde(default = "Default::default")]
    pub redis: Redis,
    /// Mail configuration.
    pub mail: Mail,
    /// Secure configuration.
    #[serde(default = "Default::default")]
    pub secure: Secure,
}

fn default_version() -> u8 {
    CONFIG_VERSION
}

fn default_host() -> String {
    "127.0.0.1".into()
}

fn default_port() -> u32 {
    7817
}
