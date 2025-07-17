use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    internal::{serializer::common::ResponseCode, utils},
    models::users,
    state::AppState,
};

pub struct LoginAccess;

impl FromRequestParts<AppState> for LoginAccess {
    type Rejection = ResponseCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
            return Err(ResponseCode::InternalError);
        };

        let conn = &state.db;

        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let Some(session) = jar.get("session") else {
            return Err(ResponseCode::Unauthorized);
        };
        let session = session.value();
        let Ok(session) = redis.get::<_, String>(format!("session-{session}")).await else {
            return Err(ResponseCode::Unauthorized);
        };

        let Some(session) = utils::session::parse(&session) else {
            return Err(ResponseCode::Unauthorized);
        };

        if !session.validate() || users::get_user_by_id(conn, session.uid).await.is_err() {
            return Err(ResponseCode::Unauthorized);
        }

        Ok(LoginAccess)
    }
}
