use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        error::{AppError, AppErrorKind},
        session::SessionService,
    },
    models::users,
};

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

        let session_id = SessionService::create(redis, user.uid)
            .await
            .map_err(|err| {
                err.with_detail(format!(
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
