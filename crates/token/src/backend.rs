use redis::{AsyncCommands, aio::MultiplexedConnection};

use crate::TokenError;

pub struct RedisTokenBackend;

impl RedisTokenBackend {
    fn key(prefix: &str, token: &str) -> String {
        format!("{prefix}::{token}")
    }

    pub async fn issue_raw(
        redis: &mut MultiplexedConnection,
        prefix: &str,
        payload: &str,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let token = uuid::Uuid::new_v4().to_string();
        redis
            .set_ex::<_, _, ()>(Self::key(prefix, &token), payload, ttl_secs)
            .await?;
        Ok(token)
    }

    pub async fn get_raw(
        redis: &mut MultiplexedConnection,
        prefix: &str,
        token: &str,
    ) -> Result<Option<String>, TokenError> {
        Ok(redis
            .get::<_, Option<String>>(Self::key(prefix, token))
            .await?)
    }

    pub async fn consume_raw(
        redis: &mut MultiplexedConnection,
        prefix: &str,
        token: &str,
    ) -> Result<Option<String>, TokenError> {
        Ok(redis::cmd("GETDEL")
            .arg(Self::key(prefix, token))
            .query_async(redis)
            .await?)
    }

    pub async fn revoke(
        redis: &mut MultiplexedConnection,
        prefix: &str,
        token: &str,
    ) -> Result<(), TokenError> {
        redis.del::<_, usize>(Self::key(prefix, token)).await?;
        Ok(())
    }
}
