use axum::extract::{Path, State};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    internal::{
        extractor::{Json, LoginAccess, OperatorAccess},
        serializer::{Response, ResponseCode},
        utils::cookie::CookieJarExt,
    },
    services::users,
    state::AppState,
};

/// User info
pub async fn info(
    State(state): State<AppState>,
    login: LoginAccess,
) -> Response<users::InfoResponse> {
    let service = users::InfoService;
    let (user, info, settings) = login.user;

    service.info(&state.db, user, info, settings, None).await
}

/// User info by uid
pub async fn info_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response<users::InfoResponse> {
    let service = users::InfoService;

    service.info_by_uid(&state.db, uid, login.user.0).await
}

/// User delete
pub async fn delete(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<users::DeleteService>,
) -> (CookieJar, Response) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    let session = login.session;

    let res = req.delete(&state.db, login.user.0).await;

    if res.is_err() {
        return (jar, res);
    }

    if redis
        .del::<_, String>(format!("session::{session}"))
        .await
        .is_err()
    {
        return (jar, ResponseCode::InternalError.into());
    }

    let jar = jar.remove_session_cookie();

    (jar, res)
}

/// User delete by uid
pub async fn delete_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    let service = users::AdminDeleteService;

    service.delete(&state.db, uid, login.level).await
}

/// User update
pub async fn update(
    login: LoginAccess,
    State(state): State<AppState>,
    Json(req): Json<users::UpdateService>,
) -> Response {
    req.update(&state.db, login.user.0, login.user.1).await
}

/// User update by uid
pub async fn update_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
    Json(req): Json<users::UpdateService>,
) -> Response {
    req.update_by_uid(&state.db, uid, login.level).await
}
