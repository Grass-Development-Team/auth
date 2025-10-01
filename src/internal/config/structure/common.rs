use crate::internal::config::common::CONFIG_VERSION;
use crate::internal::config::{Database, Mail, Redis, Secure, Site};

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
    /// Server domain.
    /// Default domain is "http://localhost:7817".
    #[serde(default = "default_domain")]
    pub domain: String,
    /// Development mode.
    /// Default is false.
    #[serde(default)]
    pub dev_mode: bool,
    /// Site configuration.
    #[serde(default)]
    pub site: Site,
    /// Database type.
    /// Default type is "sqlite".
    #[serde(default)]
    pub database: Database,
    /// Redis configuration.
    #[serde(default)]
    pub redis: Redis,
    /// Secure configuration.
    #[serde(default)]
    pub secure: Secure,
    /// Mail configuration.
    pub mail: Option<Mail>,
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

fn default_domain() -> String {
    "http://localhost:7817".into()
}
