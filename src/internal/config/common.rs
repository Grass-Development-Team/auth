use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::warn;

use super::{Config, Mail};

/// Current configuration version.
pub const CONFIG_VERSION: u8 = 1;

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
        // if self.host.is_none() {
        //     self.host = Default::default();
        // }

        // Check if port is set or out of range (1..=65535), if not then set it to 7817
        if !(1..=65535).contains(&self.port) {
            warn!(
                "Port number {} is out of range, setting port number to default value (7817)",
                &self.port
            );
            self.port = 7817;
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
            host: "0.0.0.0".into(),
            port: 7817,
            database: Default::default(),
            redis: Default::default(),
            mail: Mail {
                host: "smtp.example.com".into(),
                port: 587,
                username: "user@example.com".into(),
                password: "PassWord".into(),
                tls: true,
            },
            secure: Default::default(),
        }
    }
}
