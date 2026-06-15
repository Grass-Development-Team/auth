pub use crate::drivers::{moka, redis};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("redis cache driver error: {0}")]
    Redis(#[from] redis::Error),
    #[error("moka cache driver error: {0}")]
    Moka(#[from] moka::Error),
    #[error("cache backend error: {0}")]
    Backend(#[from] anyhow::Error),
    #[error("cache transaction conflict: retries exhausted")]
    Conflict,
}

pub type CacheError = Error;
