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
    /// Database to use.
    /// Default Type is "sqlite".
    pub database: Option<DatabaseType>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DatabaseType {
    #[serde(rename = "postgresql")]
    Postgresql(Database),
    #[serde(rename = "mysql")]
    Mysql(Database),
    #[serde(rename = "sqlite")]
    Sqlite(DatabaseSqlite),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub dbname: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseSqlite {
    pub file: String,
}