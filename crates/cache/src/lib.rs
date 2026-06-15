//! Cache abstraction with Redis and in-process Moka backends.
//!
//! The crate exposes a small asynchronous key-value API with per-entry TTL
//! and a closure-based transaction API. Use [`Cache`] when callers should be
//! agnostic to the selected backend, or use [`drivers`] directly when a
//! concrete backend is required.

pub mod drivers;
mod error;
mod tx;

pub use crate::{
    drivers::{moka::MokaCache, redis::RedisCache},
    error::{CacheError, Error},
    tx::{CacheWrite, Tx, TxFuture, TxReader},
};

/// A cache backend selected at runtime.
pub enum Cache {
    /// Redis-backed cache storage.
    Redis(RedisCache),
    /// In-process Moka-backed cache storage.
    Moka(MokaCache),
}

impl Cache {
    /// Returns the string value for `key`, or `None` if missing/expired.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        match self {
            Cache::Redis(c) => c.get(key).await.map_err(Into::into),
            Cache::Moka(c) => c.get(key).await.map_err(Into::into),
        }
    }

    /// Stores `val` at `key` with an expiration of `ttl_secs` seconds.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn set_ex(&self, key: &str, val: &str, ttl_secs: u64) -> Result<(), CacheError> {
        match self {
            Cache::Redis(c) => c.set_ex(key, val, ttl_secs).await.map_err(Into::into),
            Cache::Moka(c) => c.set_ex(key, val, ttl_secs).await.map_err(Into::into),
        }
    }

    /// Deletes `key` if it exists.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn del(&self, key: &str) -> Result<(), CacheError> {
        match self {
            Cache::Redis(c) => c.del(key).await.map_err(Into::into),
            Cache::Moka(c) => c.del(key).await.map_err(Into::into),
        }
    }

    /// Returns the value for `key` and atomically removes it.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError> {
        match self {
            Cache::Redis(c) => c.get_del(key).await.map_err(Into::into),
            Cache::Moka(c) => c.get_del(key).await.map_err(Into::into),
        }
    }

    /// Returns the remaining TTL in seconds, or `None` if the key is absent.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, CacheError> {
        match self {
            Cache::Redis(c) => c.ttl(key).await.map_err(Into::into),
            Cache::Moka(c) => c.ttl(key).await.map_err(Into::into),
        }
    }

    /// Executes a closure-based transaction over the watched keys.
    ///
    /// Reads through [`Tx`] are immediate; writes are buffered and committed
    /// atomically when the closure returns `Ok`. The closure must be safe to
    /// call more than once because Redis conflicts cause retries.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Conflict`] when Redis retry limit is exhausted.
    /// Propagates any driver or closure error otherwise.
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
