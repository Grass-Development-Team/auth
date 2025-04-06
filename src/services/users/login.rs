use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::models::users;
use crate::models::users::AccountStatus;
use crate::services::error::ServiceError;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub uid: i32,
    pub username: String,
    pub email: String,
    pub nickname: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginService {
    pub email: String,
    pub password: String,
}

impl LoginService {
    pub async fn login(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        jar: CookieJar,
    ) -> (CookieJar, Response<LoginResponse>) {
        if let Some(session) = jar.get("session") {
            if let Ok(session) = self.validate_session(session.value(), redis).await {
                if utils::session::validate(&session) {
                    if let Ok(user) = users::get_user_by_id(conn, session.uid).await {
                        let user = user.0;
                        if user.email.eq(&self.email) {
                            if user.status == AccountStatus::Banned
                                || user.status == AccountStatus::Deleted
                            {
                                return (jar, ResponseCode::UserBlocked.into());
                            }
                            if user.status == AccountStatus::Inactive {
                                return (jar, ResponseCode::UserNotActivated.into());
                            }

                            return (
                                jar,
                                Response::new(
                                    ResponseCode::OK.into(),
                                    ResponseCode::OK.into(),
                                    Some(LoginResponse {
                                        uid: user.uid,
                                        username: user.username,
                                        email: user.email,
                                        nickname: user.nickname,
                                    }),
                                ),
                            );
                        }
                    }
                }
            }
        }

        let Ok(user) = users::get_user_by_email(conn, self.email.clone()).await else {
            return (jar, ResponseCode::UserNotFound.into());
        };

        let user = user.0;

        if !user.check_password(self.password.to_owned()) {
            return (
                jar,
                Response::new_error(
                    ResponseCode::CredentialInvalid.into(),
                    "Wrong password".into(),
                ),
            );
        }
        if user.status == AccountStatus::Banned || user.status == AccountStatus::Deleted {
            return (jar, ResponseCode::UserBlocked.into());
        }
        if user.status == AccountStatus::Inactive {
            return (jar, ResponseCode::UserNotActivated.into());
        }

        // TODO: 2-factor authentication

        let Ok(session) = self.generate_session(&user, redis).await else {
            return (jar, ResponseCode::InternalError.into());
        };

        (
            jar.add(Cookie::new("session", session)),
            Response::new(
                ResponseCode::OK.into(),
                ResponseCode::OK.into(),
                Some(LoginResponse {
                    uid: user.uid,
                    username: user.username,
                    email: user.email,
                    nickname: user.nickname,
                }),
            ),
        )
    }

    async fn generate_session(
        &self,
        users: &users::Model,
        conn: &mut MultiplexedConnection,
    ) -> Result<String, ()> {
        let session = utils::session::generate(users.uid);
        trace!("Generate session: {:?}", session);
        let Ok(session) = serde_json::to_string(&session) else {
            return Err(());
        };
        trace!("Session string: {:?}", session);
        let sid = uuid::Uuid::new_v4();
        trace!("Generate session id: {:?}", sid);

        if let Err(err) =
            conn.set(format!("session-{}", sid), session).await as Result<(), redis::RedisError>
        {
            error!("Redis error: {}", err);
            return Err(());
        };

        Ok(String::from(sid))
    }

    async fn validate_session(
        &self,
        session: &str,
        conn: &mut MultiplexedConnection,
    ) -> Result<utils::session::Session, ServiceError> {
        match conn.get::<_, String>(format!("session-{}", session)).await {
            Ok(session) => match serde_json::from_str(&session) {
                Ok(session) => Ok(session),
                Err(err) => Err(ServiceError::JSONError(err)),
            },
            Err(err) => Err(ServiceError::RedisError(err)),
        }
    }
}
