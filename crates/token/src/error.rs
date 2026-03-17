#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("token backend error: {0}")]
    Backend(#[from] redis::RedisError),
    #[error("token payload encode/decode error: {0}")]
    Payload(#[from] serde_json::Error),
}
