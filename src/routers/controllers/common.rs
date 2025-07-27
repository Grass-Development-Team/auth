use crate::internal::serializer::{Response, ResponseCode};
use axum::Json;
use axum::http::StatusCode;

pub async fn not_found() -> (StatusCode, Json<Response>) {
    (StatusCode::NOT_FOUND, Json(ResponseCode::NotFound.into()))
}

pub async fn ping() -> Json<Response> {
    Json(Response::new(ResponseCode::OK.into(), "pong".into(), None))
}
