use crate::internal::auth::LoginAccess;
use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::services::users::{
    InfoResponse, InfoService, LoginResponse, LoginService, RegisterService,
};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

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
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };
    let (jar, res) = req.login(&state.db, &mut redis, jar).await;
    (jar, Json(res))
}

/// User logout
pub async fn logout(_: LoginAccess, jar: CookieJar) -> (CookieJar, Json<Response<String>>) {
    let jar = jar.remove("session");
    (
        jar,
        Json(Response::new(
            ResponseCode::OK.into(),
            ResponseCode::OK.into(),
            None,
        )),
    )
}

/// User info
pub async fn info(
    _: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Json<Response<InfoResponse>>) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    let Some(session) = jar.get("session") else {
        let jar = jar.remove("session");
        return (jar, ResponseCode::Unauthorized.into());
    };
    let session = session.value();
    let Ok(session) = redis.get::<_, String>(format!("session-{session}")).await else {
        let jar = jar.remove("session");
        return (jar, ResponseCode::Unauthorized.into());
    };

    let Some(session) = utils::session::parse_from_str(&session) else {
        let jar = jar.remove("session");
        return (jar, ResponseCode::Unauthorized.into());
    };

    let service = InfoService;

    let res = service.info(&state.db, session.uid).await;

    (jar, Json(res))
}

pub async fn info_by_uid(
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Json<Response<InfoResponse>> {
    let service = InfoService;

    let res = service.info(&state.db, uid).await;

    Json(res)
}
