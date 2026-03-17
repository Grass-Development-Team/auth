use crypto::password::PasswordError;
use sea_orm::DbErr;
use thiserror::Error;

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
