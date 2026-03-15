pub mod controllers;
pub mod response;

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{any, delete, get, patch, post},
};
use tower::ServiceBuilder;
use tower_http::{cors, cors::CorsLayer};

use crate::{
    assets::AssetManager,
    internal::{config::Config, utils::content_type},
    middleware::permission::PermissionAccess,
    routers::controllers::{auth, common, users},
    state::AppState,
};

pub fn get_router(app: Router<AppState>, config: &Config) -> Router<AppState> {
    // CORS
    let public_cors = {
        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_origin(cors::Any);
        ServiceBuilder::new().layer(cors).into_inner()
    };
    let internal_cors = {
        let cors = CorsLayer::new().allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ]);
        ServiceBuilder::new().layer(cors).into_inner()
    };

    // Oauth
    let oauth = {
        let oauth = Router::new();
        Router::new().nest("/oauth", oauth)
    };

    let api_v1 = {
        // Auth
        let auth = {
            let route = Router::new()
                .route("/login", post(auth::login))
                .route("/logout", any(auth::logout))
                .route("/register", post(auth::register))
                .route("/forget-password", post(auth::forget_password))
                .route("/reset-password", get(auth::reset_password))
                .route(
                    "/reset-password/token",
                    patch(auth::reset_password_with_token),
                )
                .route(
                    "/reset-password/password",
                    patch(auth::reset_password_with_password).layer(PermissionAccess::any(&[
                        "user:reset_password:self",
                        "user:reset_password:other",
                    ])),
                );
            let route = Router::new().nest("/auth", route);
            if config.dev_mode {
                route.layer(public_cors.clone())
            } else {
                route.layer(internal_cors.clone())
            }
        };

        // User
        let user = {
            let route = Router::new()
                .route(
                    "/info",
                    any(users::info).layer(PermissionAccess::all(&["user:read:self"])),
                )
                .route("/info/{uid}", any(users::info_by_uid))
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
                );
            Router::new().nest("/user", route)
        };

        let common = Router::new()
            .route("/ping", any(common::ping))
            .fallback(common::not_found)
            .layer(public_cors);

        let route = Router::new().merge(auth).merge(user).merge(common);
        Router::new().nest("/v1", route)
    };

    // API
    let api = {
        let route = Router::new().merge(api_v1);
        Router::new().nest("/api", route)
    };

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
