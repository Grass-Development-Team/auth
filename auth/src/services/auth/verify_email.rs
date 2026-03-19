use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use token::{TokenStore, services::RegisterTokenService};

use crate::{
    internal::error::{AppError, AppErrorKind},
    models::users,
};

#[derive(Deserialize)]
pub struct VerifyEmailService {
    pub token: String,
}

impl VerifyEmailService {
    pub async fn verify_email(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
    ) -> Result<(), AppError> {
        let token = <RegisterTokenService as TokenStore>::get(redis, &self.token)
            .await
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.verify_email.get_token",
                    err,
                )
            })?;
        let Some(token) = token else {
            return Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.get_token",
            ));
        };

        let (user, _, _) = users::get_user_by_id(conn, token.uid)
            .await
            .map_err(|err| AppError::from(err).with_op("auth.verify_email.find_user"))?;
        if user.email != token.email {
            return Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.validate_email",
            ));
        }

        if user.status.is_inactive() {
            user.update_status(conn, users::AccountStatus::Active)
                .await
                .map_err(|err| AppError::from(err).with_op("auth.verify_email.update_status"))?;
        } else if !matches!(user.status, users::AccountStatus::Active) {
            return Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.validate_status",
            ));
        }

        if let Err(err) = RegisterTokenService::consume(redis, &self.token).await {
            tracing::warn!(
                uid = token.uid,
                token = %self.token,
                error = %err,
                "verify-email token cleanup failed after state update"
            );
        }

        Ok(())
    }
}
