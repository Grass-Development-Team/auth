use crate::internal::config::structure::DatabaseType;
use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn init(config: &DatabaseType) -> Result<DatabaseConnection, DbErr> {
    let c = match config {
        DatabaseType::Postgresql(sql) => {
            format!("postgres://{}:{}@{}:{}/{}", sql.username, sql.password, sql.host, sql.port, sql.dbname)
        }
        DatabaseType::Mysql(sql) => {
            format!("mysql://{}:{}@{}:{}/{}", sql.username, sql.password, sql.host, sql.port, sql.dbname)
        }
        DatabaseType::Sqlite(sql) => {
            format!("sqlite://{}?mode=rwc", sql.file)
        }
    };
    Database::connect(c).await
}