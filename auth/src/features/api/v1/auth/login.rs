use axum::extract::State;
use axum_extra::extract::CookieJar;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use token::services::SessionService;

use crate::{
    domain::users,
    internal::{
        error::{AppError, AppErrorKind},
        session::SESSION_TTL_SECONDS,
    },
    routers::{
        extractor::{GuestAccess, Json},
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
        utils::cookie,
    },
    state::AppState,
};

pub async fn controller(
    _: GuestAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginService>,
) -> (CookieJar, Response<LoginResponse>) {
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

/// Response structure for login API
#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub uid:      i32,
    pub username: String,
    pub email:    String,
    pub nickname: String,
}

/// Service handling user login operations
#[derive(Deserialize, Serialize)]
pub struct LoginService {
    pub email:    String,
    pub password: String,
}

impl LoginService {
    /// Main login handler - validates credentials and returns response data and
    /// session id.
    pub async fn login(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
    ) -> Result<(LoginResponse, String), AppError> {
        // Get user by email
        let Ok(user) = users::get_user_by_email(conn, &self.email).await else {
            return Err(AppError::biz(
                AppErrorKind::UserNotFound,
                "auth.login.find_user",
            ));
        };

        let user = user.0;

        if user.status.is_deleted() {
            return Err(AppError::biz(
                AppErrorKind::UserDeleted,
                "auth.login.find_user",
            ));
        }

        if user.status.is_banned() {
            return Err(AppError::biz(
                AppErrorKind::UserBlocked,
                "auth.login.check_status",
            ));
        }

        if user.status.is_inactive() {
            return Err(AppError::biz(
                AppErrorKind::UserNotActivated,
                "auth.login.check_status",
            ));
        }

        // Validate credentials and account status
        if !user.check_password(self.password.to_owned()) {
            return Err(AppError::biz(
                AppErrorKind::CredentialInvalid,
                "auth.login.verify_password",
            )
            .with_detail("Wrong password"));
        }

        // TODO: 2-factor authentication

        let session_id = SessionService::create(redis, user.uid, SESSION_TTL_SECONDS)
            .await
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.login.create_session",
                    err,
                )
                .with_detail(format!(
                    "Failed to create login session ({OP_CREATE_SESSION})",
                    OP_CREATE_SESSION = "auth.login.create_session"
                ))
            })?;

        Ok((
            LoginResponse {
                uid:      user.uid,
                username: user.username,
                email:    user.email,
                nickname: user.nickname,
            },
            session_id,
        ))
    }
}
