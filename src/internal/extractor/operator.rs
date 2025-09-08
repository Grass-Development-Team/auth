use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    internal::{extractor::LoginAccess, serializer::ResponseCode},
    state::AppState,
};

pub struct OperatorAccess(pub LoginAccess);

impl FromRequestParts<AppState> for OperatorAccess {
    type Rejection = ResponseCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let res = LoginAccess::from_request_parts(parts, state).await?;

        if res.user.0.status.is_inactive() {
            return Err(ResponseCode::UserNotActivated);
        }

        if res.user.0.status.is_banned() {
            return Err(ResponseCode::UserBlocked);
        }

        Ok(OperatorAccess(res))
    }
}
