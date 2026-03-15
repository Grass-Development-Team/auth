use std::{error::Error as StdError, fmt::Display};

use crate::routers::serializer::ResponseCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErrorKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalError,
    ParamError,
    RegistrationDisabled,
    MailServiceDisabled,
    UserNotFound,
    CredentialInvalid,
    UserBlocked,
    UserNotActivated,
    UserExists,
    AlreadyLoggedIn,
    EmailExists,
    UserDeleted,
    DuplicatePassword,
    VerificationEmailSendFailed,
}

impl From<AppErrorKind> for ResponseCode {
    fn from(value: AppErrorKind) -> Self {
        match value {
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
        }
    }
}

impl From<ResponseCode> for AppErrorKind {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::BadRequest => AppErrorKind::BadRequest,
            ResponseCode::Unauthorized => AppErrorKind::Unauthorized,
            ResponseCode::Forbidden => AppErrorKind::Forbidden,
            ResponseCode::NotFound => AppErrorKind::NotFound,
            ResponseCode::InternalError => AppErrorKind::InternalError,
            ResponseCode::ParamError => AppErrorKind::ParamError,
            ResponseCode::RegistrationDisabled => AppErrorKind::RegistrationDisabled,
            ResponseCode::MailServiceDisabled => AppErrorKind::MailServiceDisabled,
            ResponseCode::UserNotFound => AppErrorKind::UserNotFound,
            ResponseCode::CredentialInvalid => AppErrorKind::CredentialInvalid,
            ResponseCode::UserBlocked => AppErrorKind::UserBlocked,
            ResponseCode::UserNotActivated => AppErrorKind::UserNotActivated,
            ResponseCode::UserExists => AppErrorKind::UserExists,
            ResponseCode::AlreadyLoggedIn => AppErrorKind::AlreadyLoggedIn,
            ResponseCode::EmailExists => AppErrorKind::EmailExists,
            ResponseCode::UserDeleted => AppErrorKind::UserDeleted,
            ResponseCode::DuplicatePassword => AppErrorKind::DuplicatePassword,
            ResponseCode::VerificationEmailSendFailed => AppErrorKind::VerificationEmailSendFailed,
            ResponseCode::OK => AppErrorKind::InternalError,
        }
    }
}

#[derive(Debug)]
pub struct AppError {
    pub kind:   AppErrorKind,
    pub op:     &'static str,
    pub detail: Option<String>,
    source:     Option<anyhow::Error>,
}

impl AppError {
    pub fn new(kind: AppErrorKind, op: &'static str) -> Self {
        Self {
            kind,
            op,
            detail: None,
            source: None,
        }
    }

    pub fn biz(kind: AppErrorKind, op: &'static str) -> Self {
        Self::new(kind, op)
    }

    pub fn infra(kind: AppErrorKind, op: &'static str, source: impl Into<anyhow::Error>) -> Self {
        Self::new(kind, op).with_source(source)
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<anyhow::Error>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn source_ref(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|err| err.as_ref())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{:?} at {}: {}", self.kind, self.op, detail),
            None => write!(f, "{:?} at {}", self.kind, self.op),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source_ref()
    }
}
