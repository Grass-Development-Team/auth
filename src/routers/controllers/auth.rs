use axum::extract::State;
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    internal::{
        extractor::{Json, LoginAccess},
        serializer::{Response, ResponseCode},
        utils::{cookie::CookieJarExt, session},
    },
    services::auth,
    state::AppState,
};

/// User register
pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<auth::RegisterService>,
) -> Response<String> {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return ResponseCode::InternalError.into();
    };

    if let Some(s) = jar.get("session")
        && let Ok(s) = redis
            .get::<_, String>(format!("session-{}", s.value()))
            .await
        && let Some(s) = session::parse_from_str(&s)
        && s.validate()
    {
        return ResponseCode::AlreadyLoggedIn.into();
    }

    req.register(&state.db, &state.config, state.mail.as_deref())
        .await
}

/// User login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<auth::LoginService>,
) -> (CookieJar, Response<auth::LoginResponse>) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    if let Some(s) = jar.get("session")
        && let Ok(s) = redis
            .get::<_, String>(format!("session-{}", s.value()))
            .await
        && let Some(s) = session::parse_from_str(&s)
        && s.validate()
    {
        return (jar, ResponseCode::AlreadyLoggedIn.into());
    }

    let (jar, res) = req.login(&state.db, &mut redis, jar, &state.config).await;
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

    let jar = jar.remove_session_cookie();

    (
        jar,
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
    )
}

/// User reset password
pub async fn reset_password(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<auth::ResetPasswordService>,
) -> (CookieJar, Response) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };
    let session = login.session;

    let res = req.reset_password(&state.db, login.user.0).await;

    if redis
        .del::<_, String>(format!("session-{session}"))
        .await
        .is_err()
    {
        return (jar, ResponseCode::InternalError.into());
    }

    let jar = jar.remove_session_cookie();

    (jar, res)
}
