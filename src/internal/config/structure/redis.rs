use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Redis {
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for Redis {
    fn default() -> Self {
        Redis {
            host: "127.0.0.1".into(),
            port: None,
            username: None,
            password: None,
        }
    }
}
