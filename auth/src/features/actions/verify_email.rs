use anyhow::anyhow;
use assets::AssetManager;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use minijinja::{AutoEscape, Environment, context};
use serde::Deserialize;

use crate::{
    infra::{
        config::Config,
        error::{AppError, AppErrorKind},
    },
    state::AppState,
};

pub async fn controller(
    State(state): State<AppState>,
    Query(req): Query<ActionsVerifyEmailService>,
) -> Response {
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

#[derive(Deserialize)]
pub struct ActionsVerifyEmailService {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub token: String,
}

impl ActionsVerifyEmailService {
    pub fn render_verify_email_page(&self, config: &Config) -> Result<String, AppError> {
        let token = self.token.trim().to_owned();
        let initial_error = if token.is_empty() {
            String::from("Invalid verification link.")
        } else {
            String::new()
        };
        let verify_api = format!(
            "{}/api/v1/auth/verify-email",
            config.domain.trim_end_matches('/')
        );

        let file = AssetManager::get("templates/actions/verify_email.html").ok_or_else(|| {
            AppError::infra(
                AppErrorKind::InternalError,
                "actions.verify_email.load_template",
                anyhow!("templates/actions/verify_email.html not found"),
            )
        })?;
        let source = String::from_utf8(file.data.into_owned()).map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "actions.verify_email.read_template",
                err,
            )
        })?;

        let mut env = Environment::new();
        env.set_auto_escape_callback(|_| AutoEscape::Html);
        env.add_template("actions.verify-email", &source)
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "actions.verify_email.parse_template",
                    err,
                )
            })?;

        env.get_template("actions.verify-email")
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "actions.verify_email.get_template",
                    err,
                )
            })?
            .render(context! {
                token => token,
                email => self.email.clone(),
                verify_api => verify_api,
                site_name => config.site.name.clone(),
                initial_error => initial_error,
            })
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "actions.verify_email.render_template",
                    err,
                )
            })
    }
}
