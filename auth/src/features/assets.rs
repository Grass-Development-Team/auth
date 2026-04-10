use assets::AssetManager;
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, Response, StatusCode, header},
    response::IntoResponse,
};

use crate::routers::utils::content_type;

pub async fn controller(request: Request) -> impl IntoResponse {
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
