use crate::internal::serializer::common::Response;
use crate::services::users::{LoginResponse, LoginService, RegisterService};
use crate::state::AppState;
use axum::{Extension, Json};
use axum_extra::extract::CookieJar;

/// User register
pub async fn register(
    Extension(state): Extension<AppState>,
    Json(req): Json<RegisterService>,
) -> Json<Response<String>> {
    Json(req.register(&state.db).await)
}

/// User login
pub async fn login(
    Extension(mut state): Extension<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginService>,
) -> (CookieJar, Json<Response<LoginResponse>>) {
    let (jar, res) = req
        .login(&state.config, &state.db, &mut state.redis, jar)
        .await;
    (jar, Json(res))
}
