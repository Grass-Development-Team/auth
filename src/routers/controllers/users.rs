use crate::internal::serializer::common::Response;
use crate::services::users::{LoginService, RegisterService};
use crate::state::AppState;
use axum::{Extension, Json};

/// User register
pub async fn register(Extension(state): Extension<AppState>, Json(req): Json<RegisterService>) -> Json<Response<String>> {
    Json(req.register(&state.db).await)
}

/// User login
pub async fn login(Extension(state): Extension<AppState>, Json(req): Json<LoginService>) -> Json<Response<String>> {
    Json(req.login(&state.db).await)
}