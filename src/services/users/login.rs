use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::internal::utils::session::Session;
use crate::models::users;
use crate::models::users::AccountStatus;
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
    pub async fn login(&self, conn: &DatabaseConnection) -> Response<LoginResponse> {
        // Check for existing valid session
        // if let Some(session) = jar.get("session") {
        //     if let Ok(session) = self.validate_session(session.value(), redis).await {
        //         if let Some(res) = self.response_from_session(session, conn).await {
        //             return (jar, res);
        //         }
        //     }
        // }

        // Get user by email
        let Ok(user) = users::get_user_by_email(conn, self.email.clone()).await else {
            return ResponseCode::UserNotFound.into();
        };

        let user = user.0;

        // Validate credentials and account status
        if !user.check_password(self.password.to_owned()) {
            return Response::new_error(
                ResponseCode::CredentialInvalid.into(),
                "Wrong password".into(),
            );
        }
        if user.status == AccountStatus::Banned || user.status == AccountStatus::Deleted {
            return ResponseCode::UserBlocked.into();
        }
        if user.status == AccountStatus::Inactive {
            return ResponseCode::UserNotActivated.into();
        }

        // TODO: 2-factor authentication

        // Create new session
        // let Ok(session) = self.generate_session(&user, redis).await else {
        //     return ResponseCode::InternalError.into();
        // };

        // Return success response with new session cookie
        Response::new(
            ResponseCode::OK.into(),
            ResponseCode::OK.into(),
            Some(LoginResponse {
                uid: user.uid,
                username: user.username,
                email: user.email,
                nickname: user.nickname,
            }),
        )
    }

    /// Validates existing session and returns user response if valid
    async fn response_from_session(
        &self,
        session: Session,
        conn: &DatabaseConnection,
    ) -> Option<Response<LoginResponse>> {
        if utils::session::validate(&session) {
            if let Ok(user) = users::get_user_by_id(conn, session.uid).await {
                let user = user.0;
                if user.email.eq(&self.email) {
                    if user.status == AccountStatus::Banned || user.status == AccountStatus::Deleted
                    {
                        return Some(ResponseCode::UserBlocked.into());
                    }
                    if user.status == AccountStatus::Inactive {
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
        }

        None
    }

    /// Generates a new session token and stores it in Redis
    async fn generate_session(
        &self,
        users: &users::Model,
        conn: &mut MultiplexedConnection,
    ) -> anyhow::Result<String> {
        let session = utils::session::generate(users.uid);
        trace!("Generate session: {:?}", session);
        let session = serde_json::to_string(&session)?;
        trace!("Session string: {:?}", session);
        let sid = uuid::Uuid::new_v4();
        trace!("Generate session id: {:?}", sid);

        if let Err(err) =
            conn.set(format!("session-{sid}"), session).await as Result<(), redis::RedisError>
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
        conn: &mut MultiplexedConnection,
    ) -> anyhow::Result<utils::session::Session> {
        let session = conn.get::<_, String>(format!("session-{session}")).await?;
        let session = serde_json::from_str(&session)?;
        Ok(session)
    }
}
