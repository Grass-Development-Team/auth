use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response as AxumResponse},
};

use crate::{services::actions::ActionsVerifyEmailService, state::AppState};

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
