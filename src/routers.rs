pub mod controllers;

use axum::Router;
use axum::http::Method;
use axum::routing::{any, post};
use tower::ServiceBuilder;
use tower_http::cors;
use tower_http::cors::CorsLayer;

// Routers
use crate::routers::controllers::common;
use crate::routers::controllers::users;
use crate::state::AppState;

pub fn get_router(app: Router<AppState>) -> Router<AppState> {
    // CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(cors::Any);
    let cors = ServiceBuilder::new().layer(cors).into_inner();

    // User
    let user = Router::new()
        .route("/login", post(users::login))
        .route("/register", post(users::register));
    let user = Router::new().nest("/user", user);

    // Oauth
    let oauth = Router::new();
    let oauth = Router::new().nest("/oauth", oauth);

    // API
    let api = Router::new()
        .merge(user)
        .merge(oauth)
        .route("/ping", any(common::ping))
        .fallback(common::not_found)
        .layer(cors);
    let api = Router::new().nest("/api", api);

    app.merge(api)
}
