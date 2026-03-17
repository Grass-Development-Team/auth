mod backend;
mod error;
pub mod services;

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use serde::{Serialize, de::DeserializeOwned};

use crate::backend::RedisTokenBackend;
pub use crate::error::TokenError;

#[async_trait]
pub trait TokenStore {
    type Payload: Serialize + DeserializeOwned + Send + Sync;
    const PREFIX: &'static str;

    async fn issue(
        redis: &mut MultiplexedConnection,
        payload: &Self::Payload,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let payload = serde_json::to_string(payload)?;
        RedisTokenBackend::issue_raw(redis, Self::PREFIX, &payload, ttl_secs).await
    }

    async fn get(
        redis: &mut MultiplexedConnection,
        token: &str,
    ) -> Result<Option<Self::Payload>, TokenError> {
        let payload = RedisTokenBackend::get_raw(redis, Self::PREFIX, token).await?;
        payload
            .map(|payload| serde_json::from_str(&payload))
            .transpose()
            .map_err(Into::into)
    }

    async fn consume(
        redis: &mut MultiplexedConnection,
        token: &str,
    ) -> Result<Option<Self::Payload>, TokenError> {
        let payload = RedisTokenBackend::consume_raw(redis, Self::PREFIX, token).await?;
        payload
            .map(|payload| serde_json::from_str(&payload))
            .transpose()
            .map_err(Into::into)
    }

    async fn revoke(redis: &mut MultiplexedConnection, token: &str) -> Result<(), TokenError> {
        RedisTokenBackend::revoke(redis, Self::PREFIX, token).await
    }
}
