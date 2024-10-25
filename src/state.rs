use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use crate::internal::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub redis: Arc<MultiplexedConnection>,
    pub config: Arc<Config>,
}