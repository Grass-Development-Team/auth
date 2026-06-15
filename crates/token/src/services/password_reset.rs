use cache::Cache;
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

    const PREFIX: &'static str = "madoka::auth::password-reset";
}

impl PasswordResetTokenService {
    pub async fn issue_for_user(
        cache: &Cache,
        uid: i32,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        <Self as TokenStore>::issue(cache, &PasswordResetToken { uid }, ttl_secs).await
    }

    pub async fn consume_uid(cache: &Cache, token: &str) -> Result<Option<i32>, TokenError> {
        let payload = <Self as TokenStore>::consume(cache, token).await?;
        Ok(payload.map(|payload| payload.uid))
    }
}
