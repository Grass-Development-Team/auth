use crate::internal::serializer::{Response, ResponseCode};
use crate::internal::utils;
use crate::internal::utils::session::Session;
use crate::models::users;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

/// Response structure for login API
#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub uid: i32,
    pub username: String,
    pub email: String,
    pub nickname: String,
}

/// Service handling user login operations
#[derive(Deserialize, Serialize)]
pub struct LoginService {
    pub email: String,
    pub password: String,
}

impl LoginService {
    /// Main login handler - validates credentials and returns session
    pub async fn login(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        jar: CookieJar,
    ) -> (CookieJar, Response<LoginResponse>) {
        // Check for existing valid session
        if let Some(session) = jar.get("session")
            && let Ok(session) = self.validate_session(session.value(), redis).await
            && let Some(res) = self.response_from_session(session, conn).await
        {
            return (jar, res);
        }

        // Get user by email
        let Ok(user) = users::get_user_by_email(conn, self.email.clone()).await else {
            return (jar, ResponseCode::UserNotFound.into());
        };

        let user = user.0;

        if user.status.is_deleted() {
            return (jar, ResponseCode::UserDeleted.into());
        }

        // Validate credentials and account status
        if !user.check_password(self.password.to_owned()) {
            return (
                jar,
                Response::new_error(
                    ResponseCode::CredentialInvalid.into(),
                    "Wrong password".into(),
                ),
            );
        }

        if user.status.is_banned() {
            return (jar, ResponseCode::UserBlocked.into());
        }

        if user.status.is_inactive() {
            return (jar, ResponseCode::UserNotActivated.into());
        }

        // TODO: 2-factor authentication

        // Create new session
        let Ok(session) = self.generate_session(&user, redis).await else {
            return (jar, ResponseCode::InternalError.into());
        };
        let jar = jar.add(Cookie::new("session", session));

        // Return success response with new session cookie
        (
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
        )
    }

    /// Validates existing session and returns user response if valid
    async fn response_from_session(
        &self,
        session: Session,
        conn: &DatabaseConnection,
    ) -> Option<Response<LoginResponse>> {
        if session.validate()
            && let Ok(user) = users::get_user_by_id(conn, session.uid).await
        {
            let user = user.0;
            if user.email.eq(&self.email) {
                if user.status.is_banned() || user.status.is_deleted() {
                    return Some(ResponseCode::UserBlocked.into());
                }
                if user.status.is_inactive() {
                    return Some(ResponseCode::UserNotActivated.into());
                }

                return Some(Response::new(
                    ResponseCode::OK.into(),
                    ResponseCode::OK.into(),
                    Some(LoginResponse {
                        uid: user.uid,
                        username: user.username,
                        email: user.email,
                        nickname: user.nickname,
                    }),
                ));
            }
        }

        None
    }

    /// Generates a new session token and stores it in Redis
    async fn generate_session(
        &self,
        users: &users::Model,
        redis: &mut MultiplexedConnection,
    ) -> anyhow::Result<String> {
        let session = utils::session::generate(users.uid);
        trace!("Generate session: {:?}", session);
        let session = serde_json::to_string(&session)?;
        trace!("Session string: {:?}", session);
        let sid = uuid::Uuid::new_v4();
        trace!("Generate session id: {:?}", sid);

        if let Err(err) =
            redis.set(format!("session-{sid}"), session).await as Result<(), redis::RedisError>
        {
            error!("Redis error: {}", err);
            return Err(err.into());
        };

        Ok(String::from(sid))
    }

    /// Validates and retrieves session data from Redis
    async fn validate_session(
        &self,
        session: &str,
        redis: &mut MultiplexedConnection,
    ) -> anyhow::Result<utils::session::Session> {
        let session = redis.get::<_, String>(format!("session-{session}")).await?;
        let session = serde_json::from_str(&session)?;
        Ok(session)
    }
}
