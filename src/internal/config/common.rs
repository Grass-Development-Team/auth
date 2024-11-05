use super::{Config, DatabaseSqlite, DatabaseType, Redis, Secure};
use crate::internal::utils;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{fs, io};
use tracing::error;
use tracing::warn;

const CONFIG_VERSION: u8 = 1;

impl Config {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let file: &Path = Path::new(path);
        let file = File::open(file);
        let mut file = match file {
            Ok(f) => f,
            Err(err) => return Err(err),
        };
        let mut config: String = String::new();
        file.read_to_string(&mut config)?;
        Ok(config.into())
    }

    pub fn check(&mut self) {
        // Check if host is set, if not then set it to "127.0.0.1"
        if self.host.is_none() {
            self.host = Default::default();
        }
        // Check if port is set, if not then set it to 7817
        if self.port.is_none() {
            self.port = Default::default();
        } else if let Some(port) = self.port {
            if !(1024..=65535).contains(&port) {
                self.port = Default::default();
                warn!("Port number {} is out of range, setting port number to default value (7817)", port);
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
        if let Some(secure) = &self.secure {
            if secure.jwt_secret.is_none() {
                self.secure = Default::default();
            }
        }
    }

    pub fn write(&self, path: &str) {
        let config = match toml::to_string_pretty(&self) {
            Ok(config) => config,
            Err(err) => {
                error!("Failed to serialize config: {}", err);
                return;
            }
        };
        fs::write(path, config).unwrap_or_else(|e| {
            error!("Failed to write config file: {}", e);
        });
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
            database: Some(
                DatabaseType::Sqlite(DatabaseSqlite { file: "auth.db".into() })
            ),
            redis: Redis {
                host: "127.0.0.1".into(),
                port: None,
                username: None,
                password: None,
            },
            secure: Some(
                Secure {
                    jwt_secret: Some(utils::rand::string(16)),
                }
            ),
        }
    }
}