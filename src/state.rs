use crate::internal::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub static APP_STATE: OnceCell<AppState> = OnceCell::const_new();

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub redis: Arc<redis::Client>,
    pub config: Config,
}
