use crate::internal::serializer::common::{Response, ResponseCode};
use axum::http::StatusCode;
use axum::Json;

pub async fn not_found() -> (StatusCode, Json<Response>) {
    (StatusCode::NOT_FOUND, Json(ResponseCode::NotFound.into()))
}

pub async fn ping() -> Json<Response> {
    Json(Response::new(ResponseCode::OK.into(), "pong".into(), None))
}