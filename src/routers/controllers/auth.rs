use axum::extract::State;
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    internal::{
        extractor::{Json, LoginAccess},
        serializer::{Response, ResponseCode},
        utils::{cookie::CookieJarExt, session},
    },
    models::users,
    services::auth,
    state::AppState,
};

/// Auth register
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
            .get::<_, String>(format!("session::{}", s.value()))
            .await
        && let Some(s) = session::parse_from_str(&s)
        && s.validate()
    {
        return ResponseCode::AlreadyLoggedIn.into();
    }

    req.register(&state.db, &state.config, state.mail.as_deref())
        .await
}

/// Auth login
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
            .get::<_, String>(format!("session::{}", s.value()))
            .await
        && let Some(s) = session::parse_from_str(&s)
        && s.validate()
    {
        return (jar, ResponseCode::AlreadyLoggedIn.into());
    }

    let (jar, res) = req.login(&state.db, &mut redis, jar, &state.config).await;
    (jar, res)
}

/// Auth logout
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
        .del::<_, String>(format!("session::{session}"))
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

/// Auth reset password
pub async fn reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<auth::ResetPasswordService>,
) -> (CookieJar, Response) {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return (jar, ResponseCode::InternalError.into());
    };

    let session = jar.get("session").map(|v| v.value().to_owned());
    let mut login_user: Option<users::Model> = None;

    if let Some(session) = &session
        && let Ok(payload) = redis.get::<_, String>(format!("session::{session}")).await
        && let Some(session) = session::parse_from_str(&payload)
        && session.validate()
        && let Ok((user, _, _)) = users::get_user_by_id(&*state.db, session.uid).await
    {
        if !user
            .check_permission(&*state.db, "user:reset_password:self")
            .await
        {
            return (jar, ResponseCode::Forbidden.into());
        }

        login_user = Some(user);
    }

    let res = req.reset_password(&state.db, &mut redis, login_user).await;
    if res.code != u16::from(ResponseCode::OK) {
        return (jar, res);
    }

    if let Some(session) = session
        && redis
            .del::<_, String>(format!("session::{session}"))
            .await
            .is_err()
    {
        return (jar, ResponseCode::InternalError.into());
    }

    let jar = jar.remove_session_cookie();

    (jar, res)
}

/// Auth forget password
pub async fn forget_password(
    State(state): State<AppState>,
    Json(req): Json<auth::ForgetPasswordService>,
) -> Response<String> {
    let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
        return ResponseCode::InternalError.into();
    };

    req.forget_password(&state.db, &mut redis, &state.config, state.mail.as_deref())
        .await
}
