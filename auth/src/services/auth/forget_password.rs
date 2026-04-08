use minijinja::context;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use token::services::PasswordResetTokenService;

use crate::{
    internal::{
        config::Config,
        error::{AppError, AppErrorKind},
        mail::Mailer,
    },
    models::users,
    services::actions::ActionsResetPasswordService,
};

const RESET_TOKEN_TTL_SECONDS: u64 = 15 * 60;

#[derive(Deserialize)]
pub struct ForgetPasswordService {
    pub email: String,
}

impl ForgetPasswordService {
    pub async fn forget_password(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        config: &Config,
        mailer: Option<&Mailer>,
    ) -> Result<String, AppError> {
        let Some(mailer) = mailer else {
            return Err(AppError::biz(
                AppErrorKind::MailServiceDisabled,
                "auth.forget_password.check_mailer",
            ));
        };

        let Ok(user) = users::get_user_by_email(conn, &self.email).await else {
            return Err(AppError::biz(
                AppErrorKind::UserNotFound,
                "auth.forget_password.find_user",
            ));
        };

        if user.0.status != users::AccountStatus::Active {
            return Err(AppError::biz(
                AppErrorKind::UserNotActivated,
                "auth.forget_password.check_user_status",
            ));
        }

        let token =
            PasswordResetTokenService::issue_for_user(redis, user.0.uid, RESET_TOKEN_TTL_SECONDS)
                .await
                .map_err(|err| {
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "auth.forget_password.issue_token",
                        err,
                    )
                })?;

        let reset_url =
            ActionsResetPasswordService::build_reset_password_action_url(config, &token);

        if let Err(err) = mailer
            .send_mail(
                &self.email,
                "Reset your password",
                "forget_password",
                context! {
                    username => user.0.username,
                    email => self.email.clone(),
                    domain => config.domain.clone(),
                    site_name => config.site.name.clone(),
                    reset_url => reset_url,
                    expires_minutes => RESET_TOKEN_TTL_SECONDS / 60,
                },
            )
            .await
        {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.forget_password.send_mail",
                err,
            ));
        }

        Ok("Reset instructions have been sent".into())
    }
}
