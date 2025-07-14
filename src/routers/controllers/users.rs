use crate::internal::serializer::common::Response;
use crate::services::users::{LoginResponse, LoginService, RegisterService};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum_extra::extract::CookieJar;

/// User register
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterService>,
) -> Json<Response<String>> {
    Json(req.register(&state.db).await)
}

/// User login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginService>,
) -> (CookieJar, Json<Response<LoginResponse>>) {
    let mut redis = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();
    let (jar, res) = req.login(&state.db, &mut redis, jar).await;
    (jar, Json(res))
}
