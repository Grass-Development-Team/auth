use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    internal::{
        serializer::ResponseCode,
        utils::{self},
    },
    models::{role, user_info, users},
    state::AppState,
};

pub struct LoginAccess {
    pub session: String,
    pub user: (users::Model, Vec<user_info::Model>),
    pub level: i32,
}

impl FromRequestParts<AppState> for LoginAccess {
    type Rejection = ResponseCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
            return Err(ResponseCode::InternalError);
        };

        let conn = &*state.db;

        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ResponseCode::InternalError)?;

        let Some(session_cookie) = jar.get("session") else {
            return Err(ResponseCode::Unauthorized);
        };
        let session_str = session_cookie.value().to_owned();
        let Ok(session) = redis
            .get::<_, String>(format!("session-{session_str}"))
            .await
        else {
            return Err(ResponseCode::Unauthorized);
        };

        let Some(session) = utils::session::parse_from_str(&session) else {
            return Err(ResponseCode::Unauthorized);
        };

        if !session.validate() {
            return Err(ResponseCode::Unauthorized);
        }

        let user = users::get_user_by_id(conn, session.uid).await;

        let Ok(user) = user else {
            return Err(ResponseCode::Unauthorized);
        };

        if user.0.status.is_deleted() {
            return Err(ResponseCode::UserDeleted);
        }

        let Ok(level) = role::get_user_role_level(conn, user.0.uid).await else {
            return Err(ResponseCode::InternalError);
        };

        Ok(LoginAccess {
            session: session_str,
            user,
            level,
        })
    }
}
