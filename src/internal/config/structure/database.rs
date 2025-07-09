use serde::{Deserialize, Serialize};

/// Database configuration for MySQL & PostgreSQL.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub username: String,
    pub password: String,
}

impl Default for Database {
    fn default() -> Self {
        Database {
            host: "localhost".into(),
            port: 5432,
            dbname: "postgres".into(),
            username: "postgres".into(),
            password: "".into(),
        }
    }
}
