pub mod controllers;

use axum::Router;
use axum::http::Method;
use axum::routing::{any, delete, post};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::{cors, services};

use crate::internal::config::Config;
use crate::middleware::permission::PermissionAccess;

// Routers
use crate::routers::controllers::common;
use crate::routers::controllers::users;
use crate::state::AppState;

pub fn get_router(app: Router<AppState>, config: &Config) -> Router<AppState> {
    // CORS
    let api_cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(cors::Any);
    let api_cors = ServiceBuilder::new().layer(api_cors).into_inner();
    let internal_cors = CorsLayer::new().allow_methods([Method::GET, Method::POST]);
    let internal_cors = ServiceBuilder::new().layer(internal_cors).into_inner();

    // User
    let user = Router::new()
        .route("/login", post(users::login))
        .route("/register", post(users::register))
        .route(
            "/info",
            any(users::info).layer(PermissionAccess::all(&["user:read:self"])),
        )
        .route(
            "/info/{uid}",
            any(users::info_by_uid).layer(PermissionAccess::any(&[
                "user:read:active",
                "user:read:all",
            ])),
        )
        .route("/logout", any(users::logout))
        .route(
            "/delete",
            delete(users::delete).layer(PermissionAccess::all(&["user:delete:self"])),
        )
        .route(
            "/delete/{uid}",
            delete(users::delete).layer(PermissionAccess::all(&["user:delete:all"])),
        );
    let user = Router::new().nest("/user", user);
    let user = if config.dev_mode {
        user.layer(api_cors.clone())
    } else {
        user.layer(internal_cors.clone())
    };

    // Oauth
    let oauth = Router::new();
    let oauth = Router::new().nest("/oauth", oauth);

    // API
    let common = Router::new()
        .route("/ping", any(common::ping))
        .fallback(common::not_found)
        .layer(api_cors);
    let api = Router::new().merge(user).merge(common);
    let api = Router::new().nest("/api", api);

    app.merge(api)
        .merge(oauth)
        .fallback_service(services::ServeDir::new("public"))
}
