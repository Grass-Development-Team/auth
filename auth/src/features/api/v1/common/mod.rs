use axum::{Router, routing::any};

use crate::{infra::http::cors, state::AppState};

pub mod not_found;
pub mod ping;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ping", any(ping::controller))
        .fallback(not_found::controller)
        .layer(cors::get_public_cors())
}
