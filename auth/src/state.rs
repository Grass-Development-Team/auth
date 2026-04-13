use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::OnceCell;

use crate::infra::{config, mailer::Mailer};

pub static APP_STATE: OnceCell<AppState> = OnceCell::const_new();

#[derive(Clone)]
pub struct AppState {
    pub db:     Arc<DatabaseConnection>,
    pub redis:  Arc<redis::Client>,
    pub config: Arc<config::Config>,
    pub mail:   Option<Arc<Mailer>>,
}
