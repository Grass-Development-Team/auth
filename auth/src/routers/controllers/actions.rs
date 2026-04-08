use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response as AxumResponse},
};
use redis::aio::MultiplexedConnection;
use token::{TokenStore, services::PasswordResetTokenService};

use crate::{
    internal::error::{AppError, AppErrorKind},
    services::actions::{ActionsResetPasswordService, ActionsVerifyEmailService},
    state::AppState,
};

pub async fn verify_email(
    State(state): State<AppState>,
    Query(req): Query<ActionsVerifyEmailService>,
) -> AxumResponse {
    match req.render_verify_email_page(&state.config) {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            let source = err.source_ref().map(ToString::to_string);
            tracing::error!(
                op = err.op,
                kind = ?err.kind,
                detail = ?err.detail,
                source = ?source,
                "failed to render verify-email action page"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    "<!doctype html><html><head><meta charset=\"UTF-8\" /><title>Verification \
                     Error</title></head><body><p>Unable to load verification page. Please try \
                     again later.</p></body></html>",
                ),
            )
                .into_response()
        },
    }
}

pub async fn reset_password(
    State(state): State<AppState>,
    Query(req): Query<ActionsResetPasswordService>,
) -> AxumResponse {
    let token_valid = match req.token() {
        Some(token) => {
            let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
                Ok(redis) => redis,
                Err(err) => {
                    return render_reset_password_error(AppError::infra(
                        AppErrorKind::InternalError,
                        "actions.reset_password.redis",
                        err,
                    ));
                },
            };

            match has_valid_reset_password_token(&mut redis, token).await {
                Ok(valid) => valid,
                Err(err) => return render_reset_password_error(err),
            }
        },
        None => req.token.is_none(),
    };

    match req.render_reset_password_page(&state.config, token_valid) {
        Ok(html) => Html(html).into_response(),
        Err(err) => render_reset_password_error(err),
    }
}

async fn has_valid_reset_password_token(
    redis: &mut MultiplexedConnection,
    token: &str,
) -> Result<bool, AppError> {
    <PasswordResetTokenService as TokenStore>::get(redis, token)
        .await
        .map(|payload| payload.is_some())
        .map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "actions.reset_password.get_token",
                err,
            )
        })
}

fn render_reset_password_error(err: AppError) -> AxumResponse {
    let source = err.source_ref().map(ToString::to_string);
    tracing::error!(
        op = err.op,
        kind = ?err.kind,
        detail = ?err.detail,
        source = ?source,
        "failed to render reset-password action page"
    );
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(
            "<!doctype html><html><head><meta charset=\"UTF-8\" /><title>Reset Password \
             Error</title></head><body><p>Unable to load reset-password page. Please try again \
             later.</p></body></html>",
        ),
    )
        .into_response()
}
