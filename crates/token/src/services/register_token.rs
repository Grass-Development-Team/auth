use cache::Cache;
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterToken {
    pub uid:   i32,
    pub email: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterTokenService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterTokenLease {
    pub token:    String,
    pub ttl_secs: u64,
}

#[async_trait::async_trait]
impl TokenStore for RegisterTokenService {
    type Payload = RegisterToken;

    const PREFIX: &'static str = "madoka::auth::register-token";
}

impl RegisterTokenService {
    const INDEX_PREFIX: &'static str = "madoka::auth::register-token-user";

    fn index_key(uid: i32) -> String {
        format!("{}::{uid}", Self::INDEX_PREFIX)
    }

    fn token_key(token: &str) -> String {
        format!("{}::{token}", Self::PREFIX)
    }

    pub async fn issue_for_user(
        cache: &Cache,
        uid: i32,
        email: impl Into<String>,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let token = RegisterToken {
            uid,
            email: email.into(),
        };
        <Self as TokenStore>::issue(cache, &token, ttl_secs).await
    }

    pub async fn issue_or_reuse_for_user(
        cache: &Cache,
        uid: i32,
        email: impl Into<String>,
        ttl_secs: u64,
        min_reuse_ttl_secs: u64,
    ) -> Result<RegisterTokenLease, TokenError> {
        let email = email.into();
        let new_token = uuid::Uuid::new_v4().to_string();
        let idx_key = Self::index_key(uid);
        let prefix = Self::PREFIX;

        let watch = [idx_key.clone()];
        let lease = cache
            .transaction(&watch, move |tx| {
                let (idx_key, email, new_token) =
                    (idx_key.clone(), email.clone(), new_token.clone());
                Box::pin(async move {
                    if let Some(existing) = tx.get(&idx_key).await? {
                        let existing_key = format!("{prefix}::{existing}");
                        if let Some(payload) = tx.get(&existing_key).await? {
                            if let Ok(decoded) = serde_json::from_str::<RegisterToken>(&payload) {
                                let ttl = tx.ttl(&existing_key).await?.unwrap_or(0);
                                if decoded.uid == uid
                                    && decoded.email == email
                                    && ttl > min_reuse_ttl_secs as i64
                                {
                                    tx.set_ex(&idx_key, existing.clone(), ttl as u64);
                                    return Ok(RegisterTokenLease {
                                        token:    existing,
                                        ttl_secs: ttl as u64,
                                    });
                                }
                            }
                            tx.del(&existing_key);
                        }
                    }

                    let payload = serde_json::to_string(&RegisterToken {
                        uid,
                        email: email.clone(),
                    })
                    .map_err(|e| cache::CacheError::Backend(e.into()))?;
                    let new_key = format!("{prefix}::{new_token}");
                    tx.set_ex(&new_key, payload, ttl_secs);
                    tx.set_ex(&idx_key, new_token.clone(), ttl_secs);
                    Ok(RegisterTokenLease {
                        token: new_token,
                        ttl_secs,
                    })
                })
            })
            .await?;
        Ok(lease)
    }

    pub async fn consume(cache: &Cache, token: &str) -> Result<Option<RegisterToken>, TokenError> {
        let Some(payload) = cache.get_del(&Self::token_key(token)).await? else {
            return Ok(None);
        };
        let decoded: RegisterToken = serde_json::from_str(&payload)?;
        let idx_key = Self::index_key(decoded.uid);
        let token = token.to_owned();
        let watch = [idx_key.clone()];
        cache
            .transaction(&watch, move |tx| {
                let (idx_key, token) = (idx_key.clone(), token.clone());
                Box::pin(async move {
                    if tx.get(&idx_key).await?.as_deref() == Some(token.as_str()) {
                        tx.del(&idx_key);
                    }
                    Ok(())
                })
            })
            .await?;
        Ok(Some(decoded))
    }
}
