use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Version of the config file.
    pub version: u8,
    /// Hostname of the server.
    /// Default is "127.0.0.1".
    pub host: Option<String>,
    /// Port of the server.
    /// Default is 7817.
    pub port: Option<u32>,
}