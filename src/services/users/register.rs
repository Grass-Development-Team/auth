use std::sync::OnceLock;

use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::internal::validator::Validatable;
use crate::models::users::AccountStatus;
use crate::models::{user_info, users};
use regex::Regex;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tracing::error;

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

        let user = users::ActiveModel {
            username: Set(self.username.to_owned()),
            email: Set(self.email.to_owned()),
            password: Set(format!("sha2:{password}:{salt}")),
            nickname: Set(if self.nickname.is_some() {
                self.nickname.to_owned().unwrap()
            } else {
                self.email.split("@").collect::<Vec<&str>>()[0].to_owned()
            }),
            status: Set(AccountStatus::Inactive),
            ..Default::default()
        };

        // Insert User
        let user = user.insert(conn).await;
        if let Err(err) = user {
            error!("Database Error: {}", err);
            return ResponseCode::InternalError.into();
        }
        let user = user.unwrap();

        // Insert User Info
        let info = user_info::ActiveModel {
            uid: Set(user.uid),
            ..Default::default()
        };

        let info = info.insert(conn).await;
        if let Err(err) = info {
            error!("Database Error:  {}", err);
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
