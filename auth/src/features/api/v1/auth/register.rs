use std::sync::OnceLock;

use axum::extract::State;
use crypto::password::{PasswordHashAlgorithm, PasswordManager};
use minijinja::context;
use redis::aio::MultiplexedConnection;
use regex::Regex;
use sea_orm::{DatabaseConnection, TransactionError, TransactionTrait};
use serde::Deserialize;
use token::services::RegisterTokenService;
use validator::Validatable;

use crate::{
    internal::{
        config::Config,
        error::{AppError, AppErrorKind},
        mail::Mailer,
    },
    models::{common::ModelError, users},
    routers::{
        extractor::{GuestAccess, Json},
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
    },
    state::AppState,
};

pub async fn controller(
    _guest: GuestAccess,
    State(state): State<AppState>,
    Json(req): Json<RegisterService>,
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

static EMAIL_RE: OnceLock<Regex> = OnceLock::new();
const REGISTER_TOKEN_TTL_SECONDS: u64 = 60 * 60;
const REGISTER_TOKEN_REUSE_MIN_TTL_SECONDS: u64 = 60;

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
        redis: Option<&mut MultiplexedConnection>,
    ) -> Result<String, AppError> {
        if !config.site.enable_registration {
            return Err(AppError::biz(
                AppErrorKind::RegistrationDisabled,
                "auth.register.check_enabled",
            ));
        }

        if mailer.is_some() && redis.is_none() {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.register.precheck_redis",
                anyhow::anyhow!("Redis connection not available"),
            )
            .with_detail("Unable to connect to redis"));
        }

        self.validate()?;

        if let Ok((user, _, _)) = users::get_user_by_email(conn, &self.email).await {
            if let Some(mailer) = mailer
                && user.status.is_inactive()
            {
                Self::send_verification_email(
                    mailer,
                    redis,
                    config,
                    user.uid,
                    &user.username,
                    &user.email,
                )
                .await?;

                return Ok("Verification email sent successfully".into());
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
        let password =
            match PasswordManager::hash(PasswordHashAlgorithm::Argon2id, &self.password, &salt) {
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
            let (user, _, _) =
                users::get_user_by_email(conn, &self.email)
                    .await
                    .map_err(|err| {
                        AppError::infra(
                            AppErrorKind::InternalError,
                            "auth.register.find_created_user",
                            err,
                        )
                    })?;

            Self::send_verification_email(
                mailer,
                redis,
                config,
                user.uid,
                &user.username,
                &user.email,
            )
            .await?;
            return Ok("Verification email sent successfully".into());
        }

        Ok("Register successfully".into())
    }

    async fn send_verification_email(
        mailer: &Mailer,
        redis: Option<&mut MultiplexedConnection>,
        config: &Config,
        uid: i32,
        username: &str,
        email: &str,
    ) -> Result<(), AppError> {
        let Some(redis) = redis else {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "auth.register.send_verification_email",
                anyhow::anyhow!("Redis connection not available"),
            ));
        };

        let token = RegisterTokenService::issue_or_reuse_for_user(
            redis,
            uid,
            email,
            REGISTER_TOKEN_TTL_SECONDS,
            REGISTER_TOKEN_REUSE_MIN_TTL_SECONDS,
        )
        .await
        .map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "auth.register.issue_or_reuse_token",
                err,
            )
        })?;
        let expires_minutes = token.ttl_secs.saturating_add(59) / 60;

        match mailer
            .send_mail(
                email,
                "Account registration received",
                "register",
                context! {
                    username => username.to_owned(),
                    email => email.to_owned(),
                    domain => config.domain.clone(),
                    site_name => config.site.name.clone(),
                    verification_token => token.token,
                    expires_minutes => expires_minutes,
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
        PasswordManager::validate(&self.password).map_err(|err| {
            AppError::biz(AppErrorKind::ParamError, "auth.register.validate")
                .with_detail(err.to_string())
        })?;

        Ok(())
    }
}
