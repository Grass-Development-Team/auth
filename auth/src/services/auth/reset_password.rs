use crypto::password::PasswordManager;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    internal::error::{AppError, AppErrorKind},
    models::users,
};

#[derive(Deserialize)]
pub struct ResetPasswordQuery {
    pub token: String,
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

        let key = format!("password-reset::{}", self.token);
        let uid: Option<i32> = match redis::cmd("GETDEL").arg(&key).query_async(redis).await {
            Ok(uid) => uid,
            Err(err) => {
                return Err(AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.reset_password.token.consume_token",
                    err,
                ));
            },
        };

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
        user: &users::Model,
    ) -> Result<(), AppError> {
        // TODO: Check permission whether user:reset_password:other or
        // user:reset_password:self

        if !user.check_password(self.old_password.clone()) {
            return Err(AppError::biz(
                AppErrorKind::Unauthorized,
                "auth.reset_password.password.verify_old_password",
            )
            .with_detail("Wrong password"));
        }

        if self.old_password == self.new_password {
            return Err(AppError::biz(
                AppErrorKind::DuplicatePassword,
                "auth.reset_password.password.check_new_password",
            ));
        }

        PasswordManager::validate(&self.new_password).map_err(|err| {
            AppError::biz(
                AppErrorKind::ParamError,
                "auth.reset_password.password.validate_password",
            )
            .with_detail(err.to_string())
        })?;

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.reset_password.password.update_password",
                err,
            ));
        }

        Ok(())
    }
}
