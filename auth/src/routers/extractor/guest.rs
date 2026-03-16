use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{
    internal::session::{SessionLookup, SessionService},
    routers::serializer::ResponseCode,
    state::AppState,
};

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

        let state = SessionService::resolve(&mut redis, session_cookie.value())
            .await
            .map_err(|_| ResponseCode::InternalError)?;

        if matches!(state, SessionLookup::Valid(_)) {
            Err(ResponseCode::AlreadyLoggedIn)
        } else {
            Ok(GuestAccess)
        }
    }
}
