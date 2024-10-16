use crate::internal::serializer::common::Response;
use crate::services::users::{LoginService, RegisterService};
use crate::state::AppState;
use axum::{Json, Extension};

/// User register
pub async fn register(Json(req): Json<RegisterService>, Extension(state): Extension<AppState>) -> Json<Response<&str>> {
    Json(req.register(&state.db))
}

/// User login
pub async fn login(Json(req): Json<LoginService>, Extension(state): Extension<AppState>) -> Json<Response> {
    todo!("Login controller");
}