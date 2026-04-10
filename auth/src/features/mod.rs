use axum::Router;

use crate::{internal::config::Config, state::AppState};

pub mod actions;
pub mod api;
pub mod assets;

pub fn router(app: Router<AppState>, config: &Config) -> Router<AppState> {
    app.merge(api::router(config))
        .merge(actions::router(config))
        .fallback(assets::controller)
}
