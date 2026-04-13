use crypto::password::PasswordError;
use sea_orm::DbErr;
use thiserror::Error;

use crate::infra::error::{AppError, AppErrorKind};

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Database error: {0}")]
    DBError(#[from] DbErr),
    #[error("Password error: {0}")]
    PasswordError(#[from] PasswordError),
    #[error("Wrong in params")]
    ParamsError,
    #[error("No entity found or empty result")]
    Empty,
    #[error("Model error: {0}")]
    Custom(String),
}

impl From<ModelError> for AppError {
    fn from(value: ModelError) -> Self {
        match value {
            ModelError::DBError(err) => AppError::new()
                .with_kind(AppErrorKind::InternalError)
                .with_source(err),
            ModelError::PasswordError(err) => AppError::new()
                .with_kind(AppErrorKind::ParamError)
                .with_detail(err.to_string()),
            ModelError::ParamsError => AppError::new().with_kind(AppErrorKind::ParamError),
            ModelError::Empty => AppError::new().with_kind(AppErrorKind::NotFound),
            ModelError::Custom(msg) => AppError::new()
                .with_kind(AppErrorKind::InternalError)
                .with_detail(msg),
        }
    }
}
