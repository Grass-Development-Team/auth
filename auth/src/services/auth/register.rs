use std::sync::OnceLock;

use crypto::password::{PasswordHashAlgorithm, PasswordManager};
use minijinja::context;
use regex::Regex;
use sea_orm::{DatabaseConnection, TransactionError, TransactionTrait};
use serde::Deserialize;
use validator::Validatable;

use crate::{
    internal::{
        config::Config,
        error::{AppError, AppErrorKind},
        mail::Mailer,
    },
    models::{common::ModelError, users},
};

static EMAIL_RE: OnceLock<Regex> = OnceLock::new();
static PASSWORD_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Deserialize)]
pub struct RegisterService {
    pub email:    String,
    pub username: String,
    pub password: String,
    pub nickname: Option<String>,
}

impl RegisterService {
    pub async fn register(
        &self,
        conn: &DatabaseConnection,
        config: &Config,
        mailer: Option<&Mailer>,
    ) -> Result<String, AppError> {
        if !config.site.enable_registration {
            return Err(AppError::biz(
                AppErrorKind::RegistrationDisabled,
                "auth.register.check_enabled",
            ));
        }

        self.validate()?;

        if let Ok(user) = users::get_user_by_email(conn, &self.email).await {
            if let Some(mailer) = mailer
                && user.0.status.is_inactive()
            {
                // TODO: Check if the verification token has expired
                return match self.send_verification_email(mailer, config).await {
                    Ok(_) => Ok("Verification email sent successfully".into()),
                    Err(err) => Err(err),
                };
            }

            return Err(AppError::biz(
                AppErrorKind::EmailExists,
                "auth.register.check_email_exists",
            ));
        }

        if users::get_user_by_username(conn, &self.username)
            .await
            .is_ok()
        {
            return Err(AppError::biz(
                AppErrorKind::UserExists,
                "auth.register.check_username_exists",
            ));
        }

        // Encrypt Password
        let salt = PasswordManager::generate_salt();
        let password = match PasswordManager::hash_password(
            PasswordHashAlgorithm::Argon2id,
            &self.password,
            &salt,
        ) {
            Ok(password) => password,
            Err(err) => {
                return Err(AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.register.hash_password",
                    err,
                ));
            },
        };

        let username = self.username.clone();
        let email = self.email.clone();
        let nickname = self.nickname.clone();

        // Check if mail service is enabled
        let status = if mailer.is_some() {
            users::AccountStatus::Inactive
        } else {
            users::AccountStatus::Active
        };

        let res: Result<_, TransactionError<ModelError>> = conn
            .transaction(|txn| {
                Box::pin(async move {
                    users::create_user(
                        txn,
                        users::CreateUserParams {
                            username,
                            email,
                            password,
                            status,
                            nickname,
                            ..Default::default()
                        },
                    )
                    .await
                })
            })
            .await;

        if let Err(err) = res {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.register.create_user",
                err,
            ));
        }

        if let Some(mailer) = mailer {
            return match self.send_verification_email(mailer, config).await {
                Ok(_) => Ok("Verification email sent successfully".into()),
                Err(err) => Err(err),
            };
        }

        Ok("Register successfully".into())
    }

    async fn send_verification_email(
        &self,
        mailer: &Mailer,
        config: &Config,
    ) -> Result<(), AppError> {
        // TODO: Send verification link
        match mailer
            .send_mail(
                &self.email,
                "Account registration received",
                "register",
                context! {
                    username => self.username.clone(),
                    email => self.email.clone(),
                    domain => config.domain.clone(),
                    site_name => config.site.name.clone(),
                },
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(AppError::infra(
                AppErrorKind::VerificationEmailSendFailed,
                "auth.register.send_verification_email",
                err,
            )
            .with_detail("Verification email could not be sent. Please try again later.")),
        }
    }
}

impl Validatable<AppError> for RegisterService {
    fn validate(&self) -> Result<(), AppError> {
        // Validate Username
        if self.username.len() < 3
            || self.username.len() > 32
            || !self
                .username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(
                AppError::biz(AppErrorKind::ParamError, "auth.register.validate").with_detail(
                    "Username must be between 3 and 32 characters and can only contain \
                     alphanumeric characters, underscores, and hyphens.",
                ),
            );
        }

        // Validate Email
        let email_re = EMAIL_RE.get_or_init(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap());
        if !email_re.is_match(&self.email) {
            return Err(
                AppError::biz(AppErrorKind::ParamError, "auth.register.validate")
                    .with_detail("Email must be a valid email address."),
            );
        }

        // Validate Password
        let password_re = PASSWORD_RE.get_or_init(|| {
            Regex::new(r#"^[A-Za-z\d!@#$%^&*()_+\-=\[\]{};':",.<>/?]{8,64}$"#).unwrap()
        });
        if !password_re.is_match(&self.password) {
            return Err(
                AppError::biz(AppErrorKind::ParamError, "auth.register.validate").with_detail(
                    "Password must be between 8 and 64 characters and contain at least one letter \
                     and one number.",
                ),
            );
        }
        if !self.password.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(
                AppError::biz(AppErrorKind::ParamError, "auth.register.validate")
                    .with_detail("Password must contain at least one letter."),
            );
        }
        if !self.password.chars().any(|c| c.is_ascii_digit()) {
            return Err(
                AppError::biz(AppErrorKind::ParamError, "auth.register.validate")
                    .with_detail("Password must contain at least one number."),
            );
        }

        Ok(())
    }
}
