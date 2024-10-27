use crate::internal::config::Config;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub redis: MultiplexedConnection,
    pub config: Config,
}