use axum::extract::State;
use axum_extra::extract::CookieJar;
use redis::aio::MultiplexedConnection;
use token::services::SessionService;

use crate::{
    internal::error::{AppError, AppErrorKind},
    routers::{
        extractor::{GuestAccess, Json, LoginAccess},
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
        utils::cookie::{self, CookieJarExt},
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
    let mut redis: Option<MultiplexedConnection> = if state.mail.is_some() {
        match state.redis.get_multiplexed_tokio_connection().await {
            Ok(redis) => Some(redis),
            Err(err) => {
                return app_error_to_response(
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "auth.controller.register.redis",
                        err,
                    )
                    .with_detail("Unable to connect to redis"),
                );
            },
        }
    } else {
        None
    };

    match req
        .register(
            &state.db,
            &state.config,
            state.mail.as_deref(),
            redis.as_mut(),
        )
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
        return (
            jar,
            app_error_to_response(AppError::infra(
                AppErrorKind::InternalError,
                "auth.controller.logout.delete_session",
                err,
            )),
        );
    }

    let jar = jar.remove_session_cookie();

    (
        jar,
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
    )
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
        return (
            jar,
            app_error_to_response(AppError::infra(
                AppErrorKind::InternalError,
                "auth.controller.reset_password_password.delete_session",
                err,
            )),
        );
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

pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<auth::VerifyEmailService>,
) -> Response {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return app_error_to_response(
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.controller.verify_email.redis",
                    err,
                )
                .with_detail("Unable to connect to redis"),
            );
        },
    };

    match req.verify_email(&state.db, &mut redis).await {
        Ok(_) => Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
        Err(err) => app_error_to_response(err),
    }
}
