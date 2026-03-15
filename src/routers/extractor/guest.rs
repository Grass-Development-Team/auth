use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{internal::session, routers::serializer::ResponseCode, state::AppState};

pub struct GuestAccess;

impl FromRequestParts<AppState> for GuestAccess {
    type Rejection = ResponseCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ResponseCode::InternalError)?;

        let Some(session_cookie) = jar.get("session") else {
            return Ok(GuestAccess);
        };

        let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
            return Err(ResponseCode::InternalError);
        };

        let Ok(payload) = redis
            .get::<_, String>(format!("session::{}", session_cookie.value()))
            .await
        else {
            return Ok(GuestAccess);
        };

        let Some(session) = session::parse_from_str(&payload) else {
            return Ok(GuestAccess);
        };

        if session.validate() {
            return Err(ResponseCode::AlreadyLoggedIn);
        }

        Ok(GuestAccess)
    }
}
