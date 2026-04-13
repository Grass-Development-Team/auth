use axum::{
    Router,
    routing::{any, patch, post},
};

use crate::{
    infra::config::Config,
    routers::{cors, middleware::permission::PermissionAccess},
    state::AppState,
};

mod forget_password;
mod login;
mod logout;
mod register;
mod reset_password;
mod verify_email;

pub fn router(config: &Config) -> Router<AppState> {
    let cors = if config.dev_mode {
        cors::get_public_cors()
    } else {
        cors::get_internal_cors()
    };

    let route = Router::new()
        .route("/login", post(login::controller))
        .route("/logout", any(logout::controller))
        .route("/register", post(register::controller))
        .route("/verify-email", post(verify_email::controller))
        .route("/forget-password", post(forget_password::controller))
        .route(
            "/reset-password/token",
            patch(reset_password::controller_with_token),
        )
        .route(
            "/reset-password/password",
            patch(reset_password::controller_with_password).layer(PermissionAccess::any(&[
                "user:reset_password:self",
                "user:reset_password:other",
            ])),
        );
    Router::new().nest("/auth", route).layer(cors)
}
