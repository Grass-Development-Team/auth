pub mod controllers;

use axum::http::Method;
use axum::routing::any;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors;
use tower_http::cors::CorsLayer;

// Routers
use crate::routers::controllers::common;

pub fn get_router(app: Router) -> Router {
    // CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(cors::Any);
    let cors = ServiceBuilder::new()
        .layer(cors)
        .into_inner();

    // User
    let user = Router::new();
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