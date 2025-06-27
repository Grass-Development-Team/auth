use serde::{Deserialize, Serialize};

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

impl Default for DatabaseType {
    fn default() -> Self {
        DatabaseType::Sqlite(DatabaseSqlite {
            file: "auth.db".into(),
        })
    }
}
