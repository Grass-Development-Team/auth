use super::structure::Config;
use std::fs::File;
use std::io::Read;
use std::path::Path;

impl Config {
    pub fn new() -> Self {
        Config {
            version: 1,
            host: Some("127.0.0.1".into()),
            port: Some(7817),
        }
    }

    pub fn from_file() -> Self {
        let file: &Path = Path::new("./config.toml");
        let file = File::open(file);
        let mut file = match file {
            Ok(f) => f,
            Err(err) => panic!("Unable to open config file: {}", err),
        };
        let mut config: String = String::new();
        if let Err(err) = file.read_to_string(&mut config) {
            panic!("{}", err);
        }
        config.into()
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
            version: 1,
            host: Some("127.0.0.1".into()),
            port: Some(7817),
        }
    }
}