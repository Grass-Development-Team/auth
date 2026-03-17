use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub uid: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PasswordResetTokenService;

#[async_trait::async_trait]
impl TokenStore for PasswordResetTokenService {
    type Payload = PasswordResetToken;

    const PREFIX: &'static str = "password-reset";
}

impl PasswordResetTokenService {
    pub async fn issue_for_user(
        redis: &mut MultiplexedConnection,
        uid: i32,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        <Self as TokenStore>::issue(redis, &PasswordResetToken { uid }, ttl_secs).await
    }

    pub async fn consume_uid(
        redis: &mut MultiplexedConnection,
        token: &str,
    ) -> Result<Option<i32>, TokenError> {
        let payload = <Self as TokenStore>::consume(redis, token).await?;
        Ok(payload.map(|payload| payload.uid))
    }
}
