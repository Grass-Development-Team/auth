use axum::http::Method;
use tower_http::{cors, cors::CorsLayer};

pub fn get_public_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_origin(cors::Any)
}

pub fn get_internal_cors() -> CorsLayer {
    CorsLayer::new().allow_methods([
        Method::GET,
        Method::POST,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
    ])
}
