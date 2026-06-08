use std::future::Future;
use std::pin::Pin;

use crate::CacheError;

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

pub struct Tx<'a> {
    reader: &'a dyn TxReader,
    writes: Vec<CacheWrite>,
}

impl<'a> Tx<'a> {
    pub fn new(reader: &'a dyn TxReader) -> Self {
        Self {
            reader,
            writes: Vec::new(),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        self.reader.get(key).await
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, CacheError> {
        self.reader.ttl(key).await
    }

    pub fn set_ex(&mut self, key: impl Into<String>, val: impl Into<String>, ttl_secs: u64) {
        self.writes.push(CacheWrite::SetEx {
            key: key.into(),
            val: val.into(),
            ttl_secs,
        });
    }

    pub fn del(&mut self, key: impl Into<String>) {
        self.writes.push(CacheWrite::Del { key: key.into() });
    }

    pub fn into_writes(self) -> Vec<CacheWrite> {
        self.writes
    }
}
