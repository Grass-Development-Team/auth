use deadpool_redis::{Config as PoolConfig, Pool, Runtime};
use redis::AsyncCommands;
use tokio::sync::Mutex;

use crate::{
    CacheError,
    tx::{CacheWrite, TxReader},
};

const MAX_TX_RETRIES: usize = 5;

pub struct RedisCache {
    pool: Pool,
}

impl RedisCache {
    pub const MAX_TX_RETRIES: usize = MAX_TX_RETRIES;

    pub fn new(url: &str) -> Result<Self, CacheError> {
        let cfg = PoolConfig::from_url(url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| CacheError::Backend(e.into()))?;
        Ok(Self { pool })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.pool.get().await?;
        Ok(conn.get::<_, Option<String>>(key).await?)
    }

    pub async fn set_ex(&self, key: &str, val: &str, ttl_secs: u64) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, _, ()>(key, val, ttl_secs).await?;
        Ok(())
    }

    pub async fn del(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    pub async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.pool.get().await?;
        Ok(redis::cmd("GETDEL")
            .arg(key)
            .query_async(&mut *conn)
            .await?)
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, CacheError> {
        let mut conn = self.pool.get().await?;
        let ttl: i64 = redis::cmd("TTL").arg(key).query_async(&mut *conn).await?;
        Ok(if ttl >= 0 { Some(ttl) } else { None })
    }

    pub async fn tx_acquire(&self, watch: &[String]) -> Result<RedisTxGuard, CacheError> {
        let mut conn = self.pool.get().await?;
        if !watch.is_empty() {
            let mut cmd = redis::cmd("WATCH");
            for k in watch {
                cmd.arg(k);
            }
            cmd.query_async::<()>(&mut *conn).await?;
        }
        Ok(RedisTxGuard {
            conn: Mutex::new(conn),
        })
    }

    pub async fn tx_commit(
        &self,
        guard: RedisTxGuard,
        writes: Vec<CacheWrite>,
    ) -> Result<bool, CacheError> {
        let mut conn = guard.conn.into_inner();
        let mut pipe = redis::pipe();
        pipe.atomic();
        for w in &writes {
            match w {
                CacheWrite::SetEx { key, val, ttl_secs } => {
                    pipe.cmd("SETEX").arg(key).arg(*ttl_secs).arg(val).ignore();
                },
                CacheWrite::Del { key } => {
                    pipe.cmd("DEL").arg(key).ignore();
                },
            }
        }
        match pipe.query_async::<()>(&mut *conn).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == redis::ErrorKind::ResponseError => Ok(false),
            Err(e) if e.kind() == redis::ErrorKind::TypeError => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct RedisTxGuard {
    conn: Mutex<deadpool_redis::Connection>,
}

impl TxReader for RedisTxGuard {
    fn get<'a>(
        &'a self,
        key: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<String>, CacheError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut conn = self.conn.lock().await;
            Ok(conn.get::<_, Option<String>>(key).await?)
        })
    }

    fn ttl<'a>(
        &'a self,
        key: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<i64>, CacheError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut conn = self.conn.lock().await;
            let ttl: i64 = redis::cmd("TTL").arg(key).query_async(&mut *conn).await?;
            Ok(if ttl >= 0 { Some(ttl) } else { None })
        })
    }
}
