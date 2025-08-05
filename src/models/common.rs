use std::error::Error;

use sea_orm::DbErr;

#[derive(Debug)]
pub enum ModelError {
    DBError(DbErr),
    Empty,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::DBError(err) => write!(f, "Database error: {}", err),
            ModelError::Empty => write!(f, "No entity found or empty result"),
        }
    }
}

impl std::error::Error for ModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ModelError::DBError(err) => Some(err),
            ModelError::Empty => None,
        }
    }
}
