use axum::extract::{Path, State};
use axum_extra::extract::CookieJar;
use crypto::password::PasswordManager;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use token::services::{PasswordResetTokenService, SessionService};

use crate::{
    domain::{role, users},
    infra::{
        error::{AppError, AppErrorKind},
        http::{
            extractor::{Json, LoginAccess, OperatorAccess},
            response::app_error_to_response,
            serializer::{Response, ResponseCode},
            utils::cookie::CookieJarExt,
        },
    },
    state::AppState,
};

pub async fn controller_with_token(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordWithTokenService>,
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

pub async fn controller_with_password(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<ResetPasswordWithPasswordService>,
) -> (CookieJar, Response) {
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

    let reset_res = req
        .reset_password(&state.db, &mut redis, &login.user.0)
        .await;

    match reset_res {
        Ok(()) => {
            let jar = jar.remove_session_cookie();
            (jar, ResponseCode::OK.into())
        },
        Err(err) => {
            let jar = if err.session_cleanup_started {
                jar.remove_session_cookie()
            } else {
                jar
            };
            (jar, app_error_to_response(err.error))
        },
    }
}

pub async fn controller_with_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
    Json(req): Json<ResetPasswordWithUidService>,
) -> Response {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return app_error_to_response(
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.controller.reset_password_password.redis",
                    err,
                )
                .with_detail("Unable to connect to redis"),
            );
        },
    };

    if let Err(err) = req
        .reset_password(&state.db, &mut redis, uid, login.level)
        .await
    {
        return app_error_to_response(err);
    }

    ResponseCode::OK.into()
}

#[derive(Deserialize)]
pub struct ResetPasswordWithTokenService {
    pub token:        String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordWithPasswordService {
    pub old_password: String,
    pub new_password: String,
}

pub struct ResetPasswordWithPasswordError {
    pub error:                   AppError,
    pub session_cleanup_started: bool,
}

#[derive(Deserialize)]
pub struct ResetPasswordWithUidService {
    pub new_password: String,
}

impl ResetPasswordWithTokenService {
    pub async fn reset_password(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
    ) -> Result<(), AppError> {
        if self.token.is_empty() || self.new_password.is_empty() {
            return Err(AppError::biz(
                AppErrorKind::ParamError,
                "auth.reset_password.token.validate_params",
            ));
        }

        PasswordManager::validate(&self.new_password).map_err(|err| {
            AppError::biz(
                AppErrorKind::ParamError,
                "auth.reset_password.token.validate_password",
            )
            .with_detail(err.to_string())
        })?;

        let uid = PasswordResetTokenService::consume_uid(redis, &self.token)
            .await
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.reset_password.token.consume_token",
                    err,
                )
            })?;

        let Some(uid) = uid else {
            return Err(AppError::biz(
                AppErrorKind::Unauthorized,
                "auth.reset_password.token.consume_token",
            ));
        };

        let Ok((user, _, _)) = users::get_user_by_id(conn, uid).await else {
            return Err(AppError::biz(
                AppErrorKind::Unauthorized,
                "auth.reset_password.token.find_user",
            ));
        };

        if user.status.is_deleted() {
            return Err(AppError::biz(
                AppErrorKind::UserDeleted,
                "auth.reset_password.token.check_user_status",
            ));
        }

        if user.check_password(self.new_password.clone()) {
            return Err(AppError::biz(
                AppErrorKind::DuplicatePassword,
                "auth.reset_password.token.check_password",
            ));
        }

        if let Err(err) = SessionService::delete_all_by_uid(redis, uid).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.reset_password.token.delete_all_sessions",
                err,
            ));
        }

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.reset_password.token.update_password",
                err,
            ));
        }

        Ok(())
    }
}

impl ResetPasswordWithPasswordService {
    pub async fn reset_password(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        user: &users::Model,
    ) -> Result<(), ResetPasswordWithPasswordError> {
        if !user.check_password(self.old_password.clone()) {
            return Err(ResetPasswordWithPasswordError {
                error: AppError::biz(
                    AppErrorKind::Unauthorized,
                    "auth.reset_password.password.verify_old_password",
                )
                .with_detail("Wrong password"),
                session_cleanup_started: false,
            });
        }

        if self.old_password == self.new_password {
            return Err(ResetPasswordWithPasswordError {
                error: AppError::biz(
                    AppErrorKind::DuplicatePassword,
                    "auth.reset_password.password.check_new_password",
                ),
                session_cleanup_started: false,
            });
        }

        PasswordManager::validate(&self.new_password).map_err(|err| {
            ResetPasswordWithPasswordError {
                error: AppError::biz(
                    AppErrorKind::ParamError,
                    "auth.reset_password.password.validate_password",
                )
                .with_detail(err.to_string()),
                session_cleanup_started: false,
            }
        })?;

        if let Err(err) = SessionService::delete_all_by_uid(redis, user.uid).await {
            return Err(ResetPasswordWithPasswordError {
                error: AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.controller.reset_password_password.delete_all_sessions",
                    err,
                ),
                session_cleanup_started: true,
            });
        }

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            return Err(ResetPasswordWithPasswordError {
                error: AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.reset_password.password.update_password",
                    err,
                ),
                session_cleanup_started: true,
            });
        }

        Ok(())
    }
}

impl ResetPasswordWithUidService {
    pub async fn reset_password(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        uid: i32,
        op_level: i32,
    ) -> Result<(), AppError> {
        let Ok((user, _, _)) = users::get_user_by_id(conn, uid).await else {
            return Err(AppError::biz(
                AppErrorKind::UserNotFound,
                "auth.reset_password.uid.find_user",
            ));
        };

        let level = role::get_user_role_level(conn, uid).await.map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "auth.reset_password.uid.role_level",
                err,
            )
        })?;

        if op_level < level {
            return Err(AppError::biz(
                AppErrorKind::Forbidden,
                "auth.reset_password.uid.check_role",
            ));
        }

        if user.status.is_deleted() {
            return Err(AppError::biz(
                AppErrorKind::UserDeleted,
                "auth.reset_password.uid.check_user_status",
            ));
        }

        if user.check_password(self.new_password.clone()) {
            return Err(AppError::biz(
                AppErrorKind::DuplicatePassword,
                "auth.reset_password.uid.check_password",
            ));
        }

        PasswordManager::validate(&self.new_password).map_err(|err| {
            AppError::biz(
                AppErrorKind::ParamError,
                "auth.reset_password.uid.validate_password",
            )
            .with_detail(err.to_string())
        })?;

        if let Err(err) = SessionService::delete_all_by_uid(redis, uid).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.reset_password.uid.delete_all_sessions",
                err,
            ));
        }

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.reset_password.uid.update_password",
                err,
            ));
        }

        Ok(())
    }
}
