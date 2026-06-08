#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache backend error: {0}")]
    Backend(#[from] anyhow::Error),
    #[error("cache transaction conflict: retries exhausted")]
    Conflict,
}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::Backend(err.into())
    }
}

impl From<deadpool_redis::PoolError> for CacheError {
    fn from(err: deadpool_redis::PoolError) -> Self {
        CacheError::Backend(err.into())
    }
}
