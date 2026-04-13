use axum::Router;

use crate::{infra::config::Config, state::AppState};

pub mod auth;
pub mod common;
pub mod users;

pub fn router(config: &Config) -> Router<AppState> {
    let route = Router::new()
        .merge(auth::router(config))
        .merge(users::router())
        .merge(common::router());
    Router::new().nest("/v1", route)
}
