use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use axum_extra::extract::CookieJar;

use crate::{
    internal::{
        error::{AppError, AppErrorKind},
        extractor::{GuestAccess, Json, LoginAccess},
        session::SessionService,
        utils::cookie::{self, CookieJarExt},
    },
    routers::{
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
    },
    services::auth,
    state::AppState,
};

/// Auth register
pub async fn register(
    _guest: GuestAccess,
    State(state): State<AppState>,
    Json(req): Json<auth::RegisterService>,
) -> Response<String> {
    match req
        .register(&state.db, &state.config, state.mail.as_deref())
        .await
    {
        Ok(message) => Response::new(
            ResponseCode::OK.into(),
            ResponseCode::OK.into(),
            Some(message),
        ),
        Err(err) => app_error_to_response(err),
    }
}

/// Auth login
pub async fn login(
    _: GuestAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<auth::LoginService>,
) -> (CookieJar, Response<auth::LoginResponse>) {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return (
                jar,
                app_error_to_response(
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "auth.controller.login.redis",
                        err,
                    )
                    .with_detail("Unable to connect to redis"),
                ),
            );
        },
    };

    match req.login(&state.db, &mut redis).await {
        Ok((data, sid)) => {
            let session_cookie = cookie::build_session_cookie(sid, !state.config.dev_mode);
            let jar = jar.add(session_cookie);

            (
                jar,
                Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(data)),
            )
        },
        Err(err) => (jar, app_error_to_response(err)),
    }
}

/// Auth logout
pub async fn logout(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Response<String>) {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return (
                jar,
                app_error_to_response(
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "auth.controller.logout.redis",
                        err,
                    )
                    .with_detail("Unable to connect to redis"),
                ),
            );
        },
    };

    let session = login.session;

    if let Err(err) = SessionService::delete(&mut redis, &session).await {
        return (jar, app_error_to_response(err));
    }

    let jar = jar.remove_session_cookie();

    (
        jar,
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
    )
}

/// Auth reset password page placeholder
pub async fn reset_password(Query(query): Query<auth::ResetPasswordQuery>) -> StatusCode {
    // TODO: Reset Password Page
    let _token = query.token;
    StatusCode::OK
}

/// Auth reset password with token
pub async fn reset_password_with_token(
    State(state): State<AppState>,
    Json(req): Json<auth::ResetPasswordWithTokenService>,
) -> Response {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return app_error_to_response(
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.controller.reset_password_token.redis",
                    err,
                )
                .with_detail("Unable to connect to redis"),
            );
        },
    };

    match req.reset_password(&state.db, &mut redis).await {
        Ok(()) => ResponseCode::OK.into(),
        Err(err) => app_error_to_response(err),
    }
}

/// Auth reset password with current password
pub async fn reset_password_with_password(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<auth::ResetPasswordWithPasswordService>,
) -> (CookieJar, Response) {
    if let Err(err) = req.reset_password(&state.db, &login.user.0).await {
        return (jar, app_error_to_response(err));
    }

    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return (
                jar,
                app_error_to_response(
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "auth.controller.reset_password_password.redis",
                        err,
                    )
                    .with_detail("Unable to connect to redis"),
                ),
            );
        },
    };

    if let Err(err) = SessionService::delete(&mut redis, &login.session).await {
        return (jar, app_error_to_response(err));
    }

    let jar = jar.remove_session_cookie();

    (jar, ResponseCode::OK.into())
}

/// Auth forget password
pub async fn forget_password(
    State(state): State<AppState>,
    Json(req): Json<auth::ForgetPasswordService>,
) -> Response<String> {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return app_error_to_response(
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.controller.forget_password.redis",
                    err,
                )
                .with_detail("Unable to connect to redis"),
            );
        },
    };

    match req
        .forget_password(&state.db, &mut redis, &state.config, state.mail.as_deref())
        .await
    {
        Ok(message) => Response::new(
            ResponseCode::OK.into(),
            ResponseCode::OK.into(),
            Some(message),
        ),
        Err(err) => app_error_to_response(err),
    }
}
