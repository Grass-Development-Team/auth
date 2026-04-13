use crate::infra::{
    error::{AppError, AppErrorKind},
    http::serializer::{Response, ResponseCode},
};

impl From<AppErrorKind> for ResponseCode {
    fn from(value: AppErrorKind) -> Self {
        match value {
            AppErrorKind::Undefined => ResponseCode::InternalError,
            AppErrorKind::BadRequest => ResponseCode::BadRequest,
            AppErrorKind::Unauthorized => ResponseCode::Unauthorized,
            AppErrorKind::Forbidden => ResponseCode::Forbidden,
            AppErrorKind::NotFound => ResponseCode::NotFound,
            AppErrorKind::InternalError => ResponseCode::InternalError,
            AppErrorKind::ParamError => ResponseCode::ParamError,
            AppErrorKind::RegistrationDisabled => ResponseCode::RegistrationDisabled,
            AppErrorKind::MailServiceDisabled => ResponseCode::MailServiceDisabled,
            AppErrorKind::UserNotFound => ResponseCode::UserNotFound,
            AppErrorKind::CredentialInvalid => ResponseCode::CredentialInvalid,
            AppErrorKind::UserBlocked => ResponseCode::UserBlocked,
            AppErrorKind::UserNotActivated => ResponseCode::UserNotActivated,
            AppErrorKind::UserExists => ResponseCode::UserExists,
            AppErrorKind::AlreadyLoggedIn => ResponseCode::AlreadyLoggedIn,
            AppErrorKind::EmailExists => ResponseCode::EmailExists,
            AppErrorKind::UserDeleted => ResponseCode::UserDeleted,
            AppErrorKind::DuplicatePassword => ResponseCode::DuplicatePassword,
            AppErrorKind::VerificationEmailSendFailed => ResponseCode::VerificationEmailSendFailed,
            AppErrorKind::TokenInvalid => ResponseCode::TokenInvalid,
        }
    }
}

pub fn app_error_to_response<T>(err: AppError) -> Response<T> {
    let code: ResponseCode = err.kind.into();
    let detail = err.detail.clone();

    let infra_kind = matches!(
        err.kind,
        AppErrorKind::InternalError | AppErrorKind::VerificationEmailSendFailed
    );

    if err.source_ref().is_some() || infra_kind {
        let source = err.source_ref().map(ToString::to_string);
        tracing::error!(
            op = err.op,
            kind = ?err.kind,
            detail = ?detail,
            source = ?source,
            "request failed with infrastructure error"
        );
    } else {
        tracing::warn!(
            op = err.op,
            kind = ?err.kind,
            detail = ?detail,
            "request failed with business error"
        );
    }

    let message = detail.unwrap_or_else(|| String::from(code));
    Response::new_error(code.into(), message)
}
