use crate::internal::config::Database as DatabaseType;
use crate::models::migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tracing::log;

pub async fn init(sql: &DatabaseType) -> Result<DatabaseConnection, DbErr> {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        sql.username, sql.password, sql.host, sql.port, sql.dbname
    );

    let mut opt = ConnectOptions::new(url);
    opt.sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}
