use axum::http::{Method, StatusCode};
use axum::{Json, Router};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::cors;
use tower_http::cors::CorsLayer;

pub fn get_router(app: Router) -> axum::Router {
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
        .fallback((
            StatusCode::NOT_FOUND,
            Json(
                json!({ "code": 404, "msg": "Not Found" })
            )
        ))
        .layer(cors);
    let api = Router::new().nest("/api", api);

    app.merge(api)
}