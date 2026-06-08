mod backend;
mod error;
pub mod services;

use async_trait::async_trait;
use cache::Cache;
use serde::{Serialize, de::DeserializeOwned};

use crate::backend::CacheTokenBackend;
pub use crate::error::TokenError;

#[async_trait]
pub trait TokenStore {
    type Payload: Serialize + DeserializeOwned + Send + Sync;
    const PREFIX: &'static str;

    async fn issue(
        cache: &Cache,
        payload: &Self::Payload,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let payload = serde_json::to_string(payload)?;
        CacheTokenBackend::issue_raw(cache, Self::PREFIX, &payload, ttl_secs).await
    }

    async fn get(cache: &Cache, token: &str) -> Result<Option<Self::Payload>, TokenError> {
        let payload = CacheTokenBackend::get_raw(cache, Self::PREFIX, token).await?;
        payload
            .map(|payload| serde_json::from_str(&payload))
            .transpose()
            .map_err(Into::into)
    }

    async fn consume(cache: &Cache, token: &str) -> Result<Option<Self::Payload>, TokenError> {
        let payload = CacheTokenBackend::consume_raw(cache, Self::PREFIX, token).await?;
        payload
            .map(|payload| serde_json::from_str(&payload))
            .transpose()
            .map_err(Into::into)
    }

    async fn revoke(cache: &Cache, token: &str) -> Result<(), TokenError> {
        CacheTokenBackend::revoke(cache, Self::PREFIX, token).await
    }
}
