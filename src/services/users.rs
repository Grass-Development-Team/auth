use crate::internal::config::Config;
use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::models::users;
use crate::models::users::{AccountPermission, AccountStatus};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginService {
    pub email: String,
    pub password: String,
}

impl LoginService {
    pub async fn login(&self, config: &Config, conn: &DatabaseConnection, redis: &mut MultiplexedConnection) -> Response<LoginResponse> {
        let Ok(user) = users::get_user_by_email(conn, self.email.clone()).await else { return ResponseCode::UserNotFound.into() };

        if !user.check_password(self.password.to_owned()) {
            return Response::new_error(ResponseCode::CredentialInvalid.into(), "Wrong password".into());
        }
        if user.status == AccountStatus::Banned || user.status == AccountStatus::Deleted {
            return ResponseCode::UserBlocked.into();
        }
        if user.status == AccountStatus::Inactive {
            return ResponseCode::UserNotActivated.into();
        }

        // TODO: 2-factor authentication

        let Some(secure) = config.secure.clone() else { return ResponseCode::InternalError.into() };
        let Some(secret) = secure.jwt_secret else { return ResponseCode::InternalError.into() };

        let Ok(token) = self.generate_token(&user, secret.as_str(), redis).await else { return ResponseCode::InternalError.into() };

        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(
            LoginResponse {
                token
            }
        ))
    }

    async fn generate_token(&self, users: &users::Model, secret: &str, conn: &mut MultiplexedConnection) -> Result<String, ()> {
        let session = utils::session::generate(users.uid);
        trace!("Generate session: {:?}", session);
        let Ok(session) = serde_json::to_string(&session) else { return Err(()) };
        trace!("Session string: {:?}", session);
        let sid = uuid::Uuid::new_v4();
        trace!("Generate session id: {:?}", sid);

        if let Err(err) = conn.set(format!("session-{}", sid), session).await as Result<(), redis::RedisError> {
            error!("Redis error: {}", err);
            return Err(());
        };

        let token = utils::jwt::generate_claim("madoka".into(), "user".into(), users.uid, sid.into());
        trace!("Generate token: {:?}", token);
        let Ok(jwt) = utils::jwt::generate(token, secret) else { return Err(()) };
        trace!("Generate jwt: {:?}", jwt);

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
            return ResponseCode::UserExists.into();
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
            return ResponseCode::InternalError.into();
        }

        // TODO: Send Verification Email

        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some("Register successfully".into()))
    }
}