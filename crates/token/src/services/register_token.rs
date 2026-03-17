use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterToken {
    pub uid:   i32,
    pub email: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterTokenService;

#[async_trait::async_trait]
impl TokenStore for RegisterTokenService {
    type Payload = RegisterToken;

    const PREFIX: &'static str = "register-token";
}

impl RegisterTokenService {
    pub async fn issue_for_user(
        redis: &mut MultiplexedConnection,
        uid: i32,
        email: impl Into<String>,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let token = RegisterToken {
            uid,
            email: email.into(),
        };
        <Self as TokenStore>::issue(redis, &token, ttl_secs).await
    }

    pub async fn consume(
        redis: &mut MultiplexedConnection,
        token: &str,
    ) -> Result<Option<RegisterToken>, TokenError> {
        <Self as TokenStore>::consume(redis, token).await
    }
}
