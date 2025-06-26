use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::warn;

use super::{Config, DatabaseSqlite, DatabaseType, Redis, Secure};
use crate::internal::utils;

/// Current configuration version.
const CONFIG_VERSION: u8 = 1;

/// Implementations of the Config struct.
impl Config {
    /// Reads the configuration file from the given path.
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let file: &Path = Path::new(path);
        let mut file = File::open(file)?;
        let mut config: String = String::new();
        file.read_to_string(&mut config)?;
        Ok(config.into())
    }

    /// Check if the configuration is valid.
    pub fn check(&mut self) {
        // Check if host is set, if not then set it to "127.0.0.1"
        if self.host.is_none() {
            self.host = Default::default();
        }

        // Check if port is set or out of range (1..=65535), if not then set it to 7817
        if self.port.is_none() {
            self.port = Default::default();
        } else if let Some(port) = self.port {
            if !(1..=65535).contains(&port) {
                self.port = Default::default();
                warn!(
                    "Port number {} is out of range, setting port number to default value (7817)",
                    port
                );
            }
        }

        // Check if database is set, if not then set it to default sqlite
        if self.database.is_none() {
            self.database = Default::default();
        }

        // Check if secure is set, if not then set it to
        if self.secure.is_none() {
            self.secure = Default::default();
        }
    }

    /// Writes the configuration file.
    pub fn write(&self, path: &str) -> anyhow::Result<()> {
        let config = match toml::to_string_pretty(&self) {
            Ok(config) => config,
            Err(err) => {
                return Err(anyhow::Error::msg(format!(
                    "Failed to serialize config: {}",
                    err
                )));
            }
        };
        fs::write(path, config)?;
        Ok(())
    }
}

impl From<&str> for Config {
    fn from(value: &str) -> Self {
        toml::from_str(value).unwrap()
    }
}

impl From<String> for Config {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<Config> for String {
    fn from(value: Config) -> Self {
        toml::to_string_pretty(&value).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CONFIG_VERSION,
            host: Some("0.0.0.0".into()),
            port: Some(7817),
            database: Some(DatabaseType::Sqlite(DatabaseSqlite {
                file: "auth.db".into(),
            })),
            redis: Redis {
                host: "127.0.0.1".into(),
                port: None,
                username: None,
                password: None,
            },
            secure: Some(Secure {
                jwt_secret: utils::rand::string(16),
            }),
        }
    }
}
