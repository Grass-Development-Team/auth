use std::{error::Error as StdError, fmt::Display};

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

#[derive(Debug)]
pub struct AppError {
    pub kind:   AppErrorKind,
    pub op:     &'static str,
    pub detail: Option<String>,
    source:     Option<anyhow::Error>,
}

impl AppError {
    pub fn new(kind: AppErrorKind) -> Self {
        Self {
            kind,
            op: "",
            detail: None,
            source: None,
        }
    }

    pub fn biz(kind: AppErrorKind, op: &'static str) -> Self {
        Self::new(kind).with_op(op)
    }

    pub fn infra(kind: AppErrorKind, op: &'static str, source: impl Into<anyhow::Error>) -> Self {
        Self::new(kind).with_op(op).with_source(source)
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<anyhow::Error>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_op(mut self, op: &'static str) -> Self {
        self.op = op;
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
