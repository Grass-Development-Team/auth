use std::{future::Future, pin::Pin, sync::Mutex};

use crate::CacheError;

/// A write operation buffered inside a transaction.
#[derive(Clone, Debug)]
pub enum CacheWrite {
    SetEx {
        key:      String,
        val:      String,
        ttl_secs: u64,
    },
    Del {
        key: String,
    },
}

/// Backend reader trait for executing immediate reads within a transaction.
pub trait TxReader: Send + Sync {
    fn get<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, CacheError>> + Send + 'a>>;

    fn ttl<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<i64>, CacheError>> + Send + 'a>>;
}

/// A boxed future returned by transaction closures.
pub type TxFuture<'t, T> = Pin<Box<dyn Future<Output = Result<T, CacheError>> + Send + 't>>;

/// Transaction handle passed to the closure in [`crate::Cache::transaction`].
///
/// Reads execute immediately against the backend. Writes are buffered and
/// committed atomically after the closure returns.
pub struct Tx<'a> {
    reader: &'a dyn TxReader,
    writes: Mutex<Vec<CacheWrite>>,
}

impl<'a> Tx<'a> {
    /// Creates a new transaction handle backed by the given reader.
    pub fn new(reader: &'a dyn TxReader) -> Self {
        Self {
            reader,
            writes: Mutex::new(Vec::new()),
        }
    }

    /// Reads `key` from the backend immediately.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        self.reader.get(key).await
    }

    /// Returns the remaining TTL for `key` in seconds.
    ///
    /// # Errors
    ///
    /// Propagates the underlying driver error.
    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, CacheError> {
        self.reader.ttl(key).await
    }

    /// Buffers a set-with-expiry write to be committed at transaction end.
    pub fn set_ex(&self, key: impl Into<String>, val: impl Into<String>, ttl_secs: u64) {
        self.writes
            .lock()
            .expect("tx writes mutex poisoned")
            .push(CacheWrite::SetEx {
                key: key.into(),
                val: val.into(),
                ttl_secs,
            });
    }

    /// Buffers a delete write to be committed at transaction end.
    pub fn del(&self, key: impl Into<String>) {
        self.writes
            .lock()
            .expect("tx writes mutex poisoned")
            .push(CacheWrite::Del { key: key.into() });
    }

    /// Takes all buffered writes out of this handle.
    pub fn take_writes(&self) -> Vec<CacheWrite> {
        std::mem::take(&mut *self.writes.lock().expect("tx writes mutex poisoned"))
    }
}
