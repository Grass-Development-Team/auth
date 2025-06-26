use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mail {
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: String,
    pub tls: bool,
}
