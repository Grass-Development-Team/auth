use crate::internal::extractor::{Json, LoginAccess};
use crate::internal::serializer::{Response, ResponseCode};
use crate::internal::utils;
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
    _: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Response<String>) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    let Some(session) = jar.get("session") else {
        return (jar, ResponseCode::Unauthorized.into());
    };
    let session = session.value();

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
    _: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response<users::InfoResponse> {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return ResponseCode::InternalError.into();
    };

    let Some(session) = jar.get("session") else {
        return ResponseCode::Unauthorized.into();
    };
    let session = session.value();
    let Ok(session) = redis.get::<_, String>(format!("session-{session}")).await else {
        return ResponseCode::InternalError.into();
    };

    let Some(session) = utils::session::parse_from_str(&session) else {
        return ResponseCode::InternalError.into();
    };

    let service = users::InfoService;

    service.info(&state.db, session.uid, session.uid).await
}

pub async fn info_by_uid(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(uid): Path<i32>,
) -> Response<users::InfoResponse> {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return ResponseCode::InternalError.into();
    };

    let Some(session) = jar.get("session") else {
        return ResponseCode::Unauthorized.into();
    };
    let session = session.value();
    let Ok(session) = redis.get::<_, String>(format!("session-{session}")).await else {
        return ResponseCode::InternalError.into();
    };

    let Some(session) = utils::session::parse_from_str(&session) else {
        return ResponseCode::InternalError.into();
    };

    let service = users::InfoService;

    service.info(&state.db, uid, session.uid).await
}

pub async fn delete(
    _: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<users::DeleteService>,
) -> (CookieJar, Response) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    let Some(session) = jar.get("session") else {
        return (jar, ResponseCode::Unauthorized.into());
    };
    let session_str = session.value();
    let Ok(session) = redis
        .get::<_, String>(format!("session-{session_str}"))
        .await
    else {
        return (jar, ResponseCode::InternalError.into());
    };

    let Some(session) = utils::session::parse_from_str(&session) else {
        return (jar, ResponseCode::InternalError.into());
    };

    let res = req.delete(&state.db, session.uid).await;

    if res.is_err() {
        return (jar, res);
    }

    if redis
        .del::<_, String>(format!("session-{session_str}"))
        .await
        .is_err()
    {
        return (jar, ResponseCode::InternalError.into());
    }

    let jar = jar.remove("session");

    (jar, res)
}

pub async fn delete_by_uid(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(uid): Path<i32>,
) -> Response {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return ResponseCode::InternalError.into();
    };

    let Some(session) = jar.get("session") else {
        return ResponseCode::Unauthorized.into();
    };
    let session_str = session.value();
    let Ok(session) = redis
        .get::<_, String>(format!("session-{session_str}"))
        .await
    else {
        return ResponseCode::InternalError.into();
    };

    let Some(session) = utils::session::parse_from_str(&session) else {
        return ResponseCode::InternalError.into();
    };

    let service = users::AdminDeleteService;

    service.delete(&state.db, uid, session.uid).await
}
