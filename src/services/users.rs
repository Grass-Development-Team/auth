use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::models::users;
use crate::models::users::{AccountPermission, AccountStatus};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Deserialize, Serialize)]
pub struct LoginService {
    pub email: String,
    pub password: String,
}

impl LoginService {
    pub async fn login(&self, conn: &DatabaseConnection) -> Response<String> {
        let Ok(user) = users::get_user_by_email(conn, self.email.clone()).await else { return Response::new_error(ResponseCode::UserNotFound.into(), ResponseCode::UserNotFound.into()) };

        if !user.check_password(self.password.to_owned()) {
            return Response::new_error(ResponseCode::CredentialInvalid.into(), "Wrong password".into());
        }
        if user.status == AccountStatus::Banned || user.status == AccountStatus::Deleted {
            return Response::new_error(ResponseCode::UserBlocked.into(), ResponseCode::UserBlocked.into());
        }
        if user.status == AccountStatus::Inactive {
            return Response::new_error(ResponseCode::UserNotActivated.into(), ResponseCode::UserNotActivated.into());
        }

        // TODO: 2-factor authentication

        // TODO: Generate Token

        // TODO: Return login info
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some("Login successfully".into()))
    }

    async fn generate_token(&self, users: &users::Model, secret: String, conn: &mut MultiplexedConnection) -> Result<String, ()> {
        let session = utils::session::generate(users.uid);
        let Ok(session) = serde_json::to_string(&session) else { return Err(()) };
        let sid = uuid::Uuid::new_v4();

        let Ok(_): Result<(), redis::RedisError> = conn.set(format!("session-{}", sid), session).await else { return Err(()) };

        let token = utils::jwt::generate_claim("madoka".into(), "user".into(), users.uid, sid.into());
        let Ok(jwt) = utils::jwt::generate(token, secret.as_ref()) else { return Err(()) };

        Ok(jwt)
    }
}

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
        if users::get_user_by_email(conn, self.email.clone()).await.is_ok() {
            return Response::new_error(ResponseCode::UserExists.into(), ResponseCode::UserExists.into());
        }

        let salt = utils::rand::string(16);
        let password = utils::password::generate(self.password.to_owned(), salt.to_owned());

        let user = users::ActiveModel {
            username: Set(self.username.to_owned()),
            email: Set(self.email.to_owned()),
            password: Set(format!("sha2:{}:{}", password, salt)),
            nickname: Set(if self.nickname.is_some() { self.nickname.to_owned().unwrap() } else { self.email.split("@").collect::<Vec<&str>>()[0].to_owned() }),
            status: Set(AccountStatus::Inactive),
            perm: Set(AccountPermission::User),
            ..Default::default()
        };

        let user = user.insert(conn).await;
        if let Err(err) = user {
            error!("Database Error: {}", err);
            return Response::new_error(ResponseCode::InternalError.into(), ResponseCode::InternalError.into());
        }

        // TODO: Send Verification Email

        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some("Register successfully".into()))
    }
}