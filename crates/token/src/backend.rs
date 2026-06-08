use cache::Cache;

use crate::TokenError;

pub struct CacheTokenBackend;

impl CacheTokenBackend {
    fn key(prefix: &str, token: &str) -> String {
        format!("{prefix}::{token}")
    }

    pub async fn issue_raw(
        cache: &Cache,
        prefix: &str,
        payload: &str,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let token = uuid::Uuid::new_v4().to_string();
        cache.set_ex(&Self::key(prefix, &token), payload, ttl_secs).await?;
        Ok(token)
    }

    pub async fn get_raw(
        cache: &Cache,
        prefix: &str,
        token: &str,
    ) -> Result<Option<String>, TokenError> {
        Ok(cache.get(&Self::key(prefix, token)).await?)
    }

    pub async fn consume_raw(
        cache: &Cache,
        prefix: &str,
        token: &str,
    ) -> Result<Option<String>, TokenError> {
        Ok(cache.get_del(&Self::key(prefix, token)).await?)
    }

    pub async fn revoke(cache: &Cache, prefix: &str, token: &str) -> Result<(), TokenError> {
        cache.del(&Self::key(prefix, token)).await?;
        Ok(())
    }
}
