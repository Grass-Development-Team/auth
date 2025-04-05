use serde::{Deserialize, Serialize};

/// Config of madoka_auth.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Config file version.
    pub version: u8,
    /// Server hostname.
    /// Default hostname is "127.0.0.1".
    pub host: Option<String>,
    /// Server port.
    /// Default port is 7817.
    pub port: Option<u32>,
    /// Database type.
    /// Default type is "sqlite".
    pub database: Option<DatabaseType>,
    /// Redis configuration.
    pub redis: Redis,
    /// Secure configuration.
    pub secure: Option<Secure>,
}

/// Database configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DatabaseType {
    #[serde(rename = "postgresql")]
    Postgresql(Database),
    #[serde(rename = "mysql")]
    Mysql(Database),
    #[serde(rename = "sqlite")]
    Sqlite(DatabaseSqlite),
}

/// Database configuration for MySQL & PostgreSQL.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub username: String,
    pub password: String,
}

/// Database configuration for SQLite.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseSqlite {
    pub file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Redis {
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Secure {
    /// JWT Secret key.
    /// If not set, a random key will be generated and stored in the config file.
    pub jwt_secret: Option<String>,
}
