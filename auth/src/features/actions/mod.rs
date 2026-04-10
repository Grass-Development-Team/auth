use axum::{Router, routing::get};

use crate::{internal::config::Config, routers::cors, state::AppState};

pub mod reset_password;
pub mod verify_email;

pub fn router(config: &Config) -> Router<AppState> {
    let cors = if config.dev_mode {
        cors::get_public_cors()
    } else {
        cors::get_internal_cors()
    };

    let route = Router::new()
        .route("/verify-email", get(verify_email::controller))
        .route("/reset-password", get(reset_password::controller));

    Router::new().nest("/actions", route).layer(cors)
}
