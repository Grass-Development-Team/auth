use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Version of the config file.
    pub version: u8,
    /// Hostname of the server.
    /// Default is "127.0.0.1".
    pub host: Option<String>,
    /// Port of the server.
    /// Default is 7817.
    pub port: Option<u32>,
    /// Database to use.
    /// Default Type is "sqlite".
    pub database: Option<DatabaseType>,
    /// Redis config of the program.
    pub redis: Redis,
    /// Secure config of the program
    pub secure: Option<Secure>,
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub username: String,
    pub password: String,
}

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