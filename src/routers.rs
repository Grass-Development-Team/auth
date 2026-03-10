pub mod controllers;

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{any, delete, patch, post},
};
use tower::ServiceBuilder;
use tower_http::{cors, cors::CorsLayer};

use crate::{
    assets::AssetManager,
    internal::{config::Config, utils::content_type},
    middleware::permission::PermissionAccess,
    routers::controllers::{common, users},
    state::AppState,
};

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
        .route("/info/{uid}", any(users::info_by_uid))
        .route("/logout", any(users::logout))
        .route(
            "/delete",
            delete(users::delete).layer(PermissionAccess::all(&["user:delete:self"])),
        )
        .route(
            "/delete/{uid}",
            delete(users::delete_by_uid).layer(PermissionAccess::all(&["user:delete:all"])),
        )
        .route(
            "/update",
            patch(users::update).layer(PermissionAccess::all(&["user:update:self"])),
        )
        .route(
            "/update/{uid}",
            patch(users::update_by_uid).layer(PermissionAccess::all(&["user:update:all"])),
        )
        .route(
            "/reset_password",
            patch(users::reset_password)
                .layer(PermissionAccess::all(&["user:reset_password:self"])),
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

    app.merge(api).merge(oauth).fallback(static_asset_fallback)
}

async fn static_asset_fallback(request: Request) -> impl IntoResponse {
    let method = request.method().clone();
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::NOT_FOUND.into_response();
    }

    let raw_path = request.uri().path().trim_start_matches('/');
    let mut candidates = Vec::new();
    if raw_path.is_empty() {
        candidates.push("public/index.html".to_owned());
    } else {
        candidates.push(format!("public/{raw_path}"));

        if raw_path.ends_with('/') || !raw_path.contains('.') {
            let trimmed = raw_path.trim_end_matches('/');
            if trimmed.is_empty() {
                candidates.push("public/index.html".to_owned());
            } else {
                candidates.push(format!("public/{trimmed}/index.html"));
            }
        }
    }

    for candidate in candidates {
        if let Some(file) = AssetManager::get(&candidate) {
            let mut response = Response::new(Body::from(file.data.into_owned()));
            *response.status_mut() = StatusCode::OK;

            if let Ok(etag) = HeaderValue::from_str(&format!(
                "\"{}\"",
                base16ct::lower::encode_string(&file.metadata.sha256_hash())
            )) {
                response.headers_mut().insert(header::ETAG, etag);
            }

            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type::from_path(&candidate)),
            );

            return response;
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
