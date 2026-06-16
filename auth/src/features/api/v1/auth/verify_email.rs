use axum::extract::State;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use token::{TokenStore, services::RegisterTokenService};

use crate::{
    domain::users,
    infra::{
        database::entity::users::AccountStatus,
        error::{AppError, AppErrorKind},
        http::{
            extractor::Json,
            response::app_error_to_response,
            serializer::{Response, ResponseCode},
        },
    },
    state::AppState,
};

pub async fn controller(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailService>,
) -> Response {
    match req.verify_email(&state.db, &state.cache).await {
        Ok(_) => Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
        Err(err) => app_error_to_response(err),
    }
}

#[derive(Deserialize)]
pub struct VerifyEmailService {
    pub token: String,
}

impl VerifyEmailService {
    pub async fn verify_email(
        &self,
        conn: &DatabaseConnection,
        cache: &cache::Cache,
    ) -> Result<(), AppError> {
        let token = <RegisterTokenService as TokenStore>::get(cache, &self.token)
            .await
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "auth.verify_email.get_token",
                    err,
                )
            })?;
        let Some(token) = token else {
            return Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.get_token",
            ));
        };

        let (user, _, _) = users::get_user_by_id(conn, token.uid)
            .await
            .map_err(|err| AppError::from(err).with_op("auth.verify_email.find_user"))?;
        if user.email != token.email {
            return Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.validate_email",
            ));
        }

        if user.status.is_inactive() {
            user.update_status(conn, AccountStatus::Active)
                .await
                .map_err(|err| AppError::from(err).with_op("auth.verify_email.update_status"))?;
        } else if !matches!(user.status, AccountStatus::Active) {
            return Err(AppError::biz(
                AppErrorKind::TokenInvalid,
                "auth.verify_email.validate_status",
            ));
        }

        if let Err(err) = RegisterTokenService::consume(cache, &self.token).await {
            tracing::warn!(
                uid = token.uid,
                token = %self.token,
                error = %err,
                "verify-email token cleanup failed after state update"
            );
        }

        Ok(())
    }
}
