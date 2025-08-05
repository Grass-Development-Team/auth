use std::sync::OnceLock;

use crate::internal::serializer::{Response, ResponseCode};
use crate::internal::utils;
use crate::internal::validator::Validatable;
use crate::models::common::ModelError;
use crate::models::users::AccountStatus;
use crate::models::{role, user_info, user_role, users};
use regex::Regex;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, TransactionError, TransactionTrait};
use serde::{Deserialize, Serialize};

static EMAIL_RE: OnceLock<Regex> = OnceLock::new();
static PASSWORD_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Deserialize, Serialize)]
pub struct RegisterService {
    pub email: String,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}

impl RegisterService {
    pub async fn register(&self, conn: &DatabaseConnection) -> Response<String> {
        if let Err(err) = self.validate() {
            return err.into();
        }

        if users::get_user_by_username(conn, self.username.clone())
            .await
            .is_ok()
        {
            return ResponseCode::UserExists.into();
        }

        if users::get_user_by_email(conn, self.email.clone())
            .await
            .is_ok()
        {
            return ResponseCode::EmailExists.into();
        }

        // Encrypt Password
        let salt = utils::rand::string(16);
        let password = utils::password::generate(self.password.to_owned(), salt.to_owned());

        let username = self.username.clone();
        let email = self.email.clone();
        let nickname = self.nickname.clone();

        let res: Result<_, TransactionError<ModelError>> = conn
            .transaction(|txn| {
                Box::pin(async move {
                    // Insert User
                    let user = users::ActiveModel {
                        username: Set(username),
                        email: Set(email.clone()),
                        password: Set(format!("sha2:{password}:{salt}")),
                        nickname: Set(if let Some(nickname) = nickname {
                            nickname
                        } else {
                            email.split("@").collect::<Vec<&str>>()[0].to_owned()
                        }),
                        status: Set(AccountStatus::Inactive),
                        ..Default::default()
                    };
                    let user = user.insert(txn).await.map_err(ModelError::DBError)?;

                    // Insert User Info
                    let info = user_info::ActiveModel {
                        uid: Set(user.uid),
                        ..Default::default()
                    };
                    info.insert(txn).await.map_err(ModelError::DBError)?;

                    // Insert User Role
                    // TODO: Default Role setting
                    let role_id = role::get_role_id(txn, "user".into()).await?;

                    let role = user_role::ActiveModel {
                        user_id: Set(user.uid),
                        role_id: Set(role_id),
                    };
                    role.insert(txn).await.map_err(ModelError::DBError)?;

                    Ok(())
                })
            })
            .await;

        if res.is_err() {
            return ResponseCode::InternalError.into();
        }

        // TODO: Send Verification Email

        Response::new(
            ResponseCode::OK.into(),
            ResponseCode::OK.into(),
            Some("Register successfully".into()),
        )
    }
}

impl Validatable for RegisterService {
    fn validate(&self) -> Result<(), ResponseCode> {
        // Validate Username
        if self.username.len() < 3
            || self.username.len() > 32
            || !self
                .username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ResponseCode::ParamError);
        }

        // Validate Email
        let email_re = EMAIL_RE.get_or_init(|| Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap());
        if !email_re.is_match(&self.email) {
            return Err(ResponseCode::ParamError);
        }

        // Validate Password
        let password_re = PASSWORD_RE.get_or_init(|| {
            Regex::new(r#"^[A-Za-z\d!@#$%^&*()_+\-=\[\]{};':",.<>/?]{8,64}$"#).unwrap()
        });
        if !password_re.is_match(&self.password) {
            return Err(ResponseCode::ParamError);
        }
        if !self.password.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(ResponseCode::ParamError);
        }
        if !self.password.chars().any(|c| c.is_ascii_digit()) {
            return Err(ResponseCode::ParamError);
        }

        Ok(())
    }
}
