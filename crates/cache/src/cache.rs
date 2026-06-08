use std::{future::Future, pin::Pin};

use crate::{
    CacheError,
    moka::MokaCache,
    redis::RedisCache,
    tx::{Tx, TxReader},
};

pub enum Cache {
    Redis(RedisCache),
    Moka(MokaCache),
}

pub type TxFuture<'t, T> = Pin<Box<dyn Future<Output = Result<T, CacheError>> + Send + 't>>;

impl Cache {
    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        match self {
            Cache::Redis(c) => c.get(key).await,
            Cache::Moka(c) => c.get(key).await,
        }
    }

    pub async fn set_ex(&self, key: &str, val: &str, ttl_secs: u64) -> Result<(), CacheError> {
        match self {
            Cache::Redis(c) => c.set_ex(key, val, ttl_secs).await,
            Cache::Moka(c) => c.set_ex(key, val, ttl_secs).await,
        }
    }

    pub async fn del(&self, key: &str) -> Result<(), CacheError> {
        match self {
            Cache::Redis(c) => c.del(key).await,
            Cache::Moka(c) => c.del(key).await,
        }
    }

    pub async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError> {
        match self {
            Cache::Redis(c) => c.get_del(key).await,
            Cache::Moka(c) => c.get_del(key).await,
        }
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, CacheError> {
        match self {
            Cache::Redis(c) => c.ttl(key).await,
            Cache::Moka(c) => c.ttl(key).await,
        }
    }

    pub async fn transaction<T, F>(&self, watch: &[String], f: F) -> Result<T, CacheError>
    where
        F: for<'t> Fn(&'t Tx<'t>) -> TxFuture<'t, T>,
        T: Send,
    {
        match self {
            Cache::Moka(c) => {
                let _guards = c.lock_for_tx(watch).await;
                let tx = Tx::new(c as &dyn TxReader);
                let out = f(&tx).await?;
                let writes = tx.take_writes();
                c.apply_writes(writes).await?;
                Ok(out)
            },
            Cache::Redis(c) => {
                for _ in 0..RedisCache::MAX_TX_RETRIES {
                    let guard = c.tx_acquire(watch).await?;
                    let tx = Tx::new(&guard as &dyn TxReader);
                    let out = f(&tx).await?;
                    let writes = tx.take_writes();
                    if c.tx_commit(guard, writes).await? {
                        return Ok(out);
                    }
                }
                Err(CacheError::Conflict)
            },
        }
    }
}
