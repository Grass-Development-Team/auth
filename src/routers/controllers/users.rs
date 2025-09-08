use crate::internal::extractor::{Json, LoginAccess, OperatorAccess};
use crate::internal::serializer::{Response, ResponseCode};
use crate::models;
use crate::services::users;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

/// User register
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<users::RegisterService>,
) -> Response<String> {
    req.register(&state.db).await
}

/// User login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<users::LoginService>,
) -> (CookieJar, Response<users::LoginResponse>) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };
    let (jar, res) = req.login(&state.db, &mut redis, jar).await;
    (jar, res)
}

/// User logout
pub async fn logout(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Response<String>) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    let session = login.session;

    if redis
        .del::<_, String>(format!("session-{session}"))
        .await
        .is_err()
    {
        return (jar, ResponseCode::InternalError.into());
    }

    let jar = jar.remove("session");

    (
        jar,
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
    )
}

/// User info
pub async fn info(
    login: LoginAccess,
    State(state): State<AppState>,
) -> Response<users::InfoResponse> {
    let service = users::InfoService;

    service.info(&state.db, login.user, None).await
}

pub async fn info_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response<users::InfoResponse> {
    let Ok(user) = models::users::get_user_by_id(&*state.db, uid).await else {
        return ResponseCode::UserNotFound.into();
    };

    let service = users::InfoService;

    service.info(&state.db, user, Some(login.user.0)).await
}

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
        .del::<_, String>(format!("session-{session}"))
        .await
        .is_err()
    {
        return (jar, ResponseCode::InternalError.into());
    }

    let jar = jar.remove("session");

    (jar, res)
}

pub async fn delete_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    let service = users::AdminDeleteService;

    service.delete(&state.db, uid, login.level).await
}
