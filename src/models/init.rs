use crate::internal::config::DatabaseType;
use crate::models::migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tracing::log;

pub async fn init(config: &DatabaseType) -> Result<DatabaseConnection, DbErr> {
    let url = match config {
        DatabaseType::Postgresql(sql) => {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                sql.username, sql.password, sql.host, sql.port, sql.dbname
            )
        }
        DatabaseType::Mysql(sql) => {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                sql.username, sql.password, sql.host, sql.port, sql.dbname
            )
        }
        DatabaseType::Sqlite(sql) => {
            format!("sqlite://{}?mode=rwc", sql.file)
        }
    };

    let mut opt = ConnectOptions::new(url);
    opt.sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}
