use axum::Router;

use crate::{infra::config::Config, state::AppState};

pub mod v1;

pub fn router(config: &Config) -> Router<AppState> {
    let route = Router::new().merge(v1::router(config));
    Router::new().nest("/api", route)
}
