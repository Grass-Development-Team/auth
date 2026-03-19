use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use token::services::RegisterTokenService;

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
        let res = RegisterTokenService::consume(redis, &self.token).await;

        match res {
            Ok(Some(token)) => {
                let (user, _, _) = users::get_user_by_id(conn, token.uid)
                    .await
                    .map_err(|err| AppError::from(err).with_op("auth.verify_email.find_user"))?;

                user.update_status(conn, users::AccountStatus::Active)
                    .await
                    .map_err(|err| {
                        AppError::from(err).with_op("auth.verify_email.update_status")
                    })?;

                Ok(())
            },
            Ok(None) => Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.consume",
            )),
            Err(err) => Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.verify_email.consume",
                err,
            )),
        }
    }
}
