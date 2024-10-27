use crate::internal::serializer::common::Response;
use crate::services::users::{LoginResponse, LoginService, RegisterService};
use crate::state::AppState;
use axum::{Extension, Json};

/// User register
pub async fn register(Extension(state): Extension<AppState>, Json(req): Json<RegisterService>) -> Json<Response<String>> {
    Json(req.register(&state.db).await)
}

/// User login
pub async fn login(Extension(mut state): Extension<AppState>, Json(req): Json<LoginService>) -> Json<Response<LoginResponse>> {
    Json(req.login(&state.config, &state.db, &mut state.redis).await)
}