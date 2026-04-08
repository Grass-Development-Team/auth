use anyhow::anyhow;
use assets::AssetManager;
use minijinja::{AutoEscape, Environment, context};
use serde::Deserialize;

use crate::internal::{
    config::Config,
    error::{AppError, AppErrorKind},
};

const ACTION_RESET_PASSWORD_PATH: &str = "/actions/reset-password";
const USER_INFO_API_PATH: &str = "/api/v1/user/info";
const RESET_PASSWORD_WITH_TOKEN_API_PATH: &str = "/api/v1/auth/reset-password/token";
const RESET_PASSWORD_WITH_PASSWORD_API_PATH: &str = "/api/v1/auth/reset-password/password";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResetPasswordPageMode {
    Token,
    SessionCheck,
    Invalid,
}

#[derive(Deserialize)]
pub struct ActionsResetPasswordService {
    #[serde(default)]
    pub token: Option<String>,
}

impl ActionsResetPasswordService {
    pub fn build_reset_password_action_url(config: &Config, token: &str) -> String {
        format!(
            "{}{}?token={token}",
            config.domain.trim_end_matches('/'),
            ACTION_RESET_PASSWORD_PATH
        )
    }

    pub fn token(&self) -> Option<&str> {
        self.token
            .as_deref()
            .map(str::trim)
            .filter(|token| !token.is_empty())
    }

    pub fn render_reset_password_page(
        &self,
        config: &Config,
        token_valid: bool,
    ) -> Result<String, AppError> {
        let mode = self.page_mode(token_valid);
        let initial_error = if mode == ResetPasswordPageMode::Invalid {
            String::from("Invalid or expired reset link.")
        } else {
            String::new()
        };
        let auth_check_api = self.auth_check_api_path(token_valid);
        let submit_api = self.submit_api_path(token_valid);
        let success_message = match mode {
            ResetPasswordPageMode::Token => {
                String::from("Password reset successfully. You can sign in with the new password.")
            },
            ResetPasswordPageMode::SessionCheck => {
                String::from("Password updated successfully. Please sign in again.")
            },
            ResetPasswordPageMode::Invalid => String::new(),
        };

        let file = AssetManager::get("templates/actions/reset_password.html").ok_or_else(|| {
            AppError::infra(
                AppErrorKind::InternalError,
                "actions.reset_password.load_template",
                anyhow!("templates/actions/reset_password.html not found"),
            )
        })?;
        let source = String::from_utf8(file.data.into_owned()).map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "actions.reset_password.read_template",
                err,
            )
        })?;

        let mut env = Environment::new();
        env.set_auto_escape_callback(|_| AutoEscape::Html);
        env.add_template("actions.reset-password", &source)
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "actions.reset_password.parse_template",
                    err,
                )
            })?;

        env.get_template("actions.reset-password")
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "actions.reset_password.get_template",
                    err,
                )
            })?
            .render(context! {
                token => self.token(),
                mode => mode.as_str(),
                auth_check_api => auth_check_api,
                submit_api => submit_api,
                site_name => config.site.name.clone(),
                initial_error => initial_error,
                success_message => success_message,
            })
            .map_err(|err| {
                AppError::infra(
                    AppErrorKind::InternalError,
                    "actions.reset_password.render_template",
                    err,
                )
            })
    }

    fn page_mode(&self, token_valid: bool) -> ResetPasswordPageMode {
        match self.token() {
            Some(_) if token_valid => ResetPasswordPageMode::Token,
            Some(_) => ResetPasswordPageMode::Invalid,
            None if self.token.is_some() => ResetPasswordPageMode::Invalid,
            None => ResetPasswordPageMode::SessionCheck,
        }
    }

    fn auth_check_api_path(&self, token_valid: bool) -> &'static str {
        match self.page_mode(token_valid) {
            ResetPasswordPageMode::SessionCheck => USER_INFO_API_PATH,
            _ => "",
        }
    }

    fn submit_api_path(&self, token_valid: bool) -> &'static str {
        match self.page_mode(token_valid) {
            ResetPasswordPageMode::Token => RESET_PASSWORD_WITH_TOKEN_API_PATH,
            ResetPasswordPageMode::SessionCheck => RESET_PASSWORD_WITH_PASSWORD_API_PATH,
            ResetPasswordPageMode::Invalid => "",
        }
    }
}

impl ResetPasswordPageMode {
    fn as_str(self) -> &'static str {
        match self {
            ResetPasswordPageMode::Token => "token",
            ResetPasswordPageMode::SessionCheck => "session-check",
            ResetPasswordPageMode::Invalid => "invalid",
        }
    }
}
