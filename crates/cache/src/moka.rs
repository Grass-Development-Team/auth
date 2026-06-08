use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::{Duration, Instant},
};

use moka::{Expiry, future::Cache as MokaMap};
use tokio::sync::Mutex;

use crate::{
    CacheError,
    tx::{CacheWrite, TxReader},
};

const STRIPES: usize = 64;

#[derive(Clone)]
struct Entry {
    value:     String,
    expire_at: Instant,
}

struct EntryExpiry;

impl Expiry<String, Entry> for EntryExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &Entry,
        created_at: Instant,
    ) -> Option<Duration> {
        Some(value.expire_at.saturating_duration_since(created_at))
    }

    fn expire_after_update(
        &self,
        _key: &String,
        value: &Entry,
        updated_at: Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        Some(value.expire_at.saturating_duration_since(updated_at))
    }
}

pub struct MokaCache {
    map:     MokaMap<String, Entry>,
    stripes: Arc<Vec<Mutex<()>>>,
}

impl MokaCache {
    pub fn new(max_capacity: u64) -> Self {
        let map = MokaMap::builder()
            .max_capacity(max_capacity)
            .expire_after(EntryExpiry)
            .build();
        let stripes = (0..STRIPES).map(|_| Mutex::new(())).collect();
        Self {
            map,
            stripes: Arc::new(stripes),
        }
    }

    fn stripe_index(key: &str) -> usize {
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        (h.finish() as usize) % STRIPES
    }

    fn live(entry: Option<Entry>) -> Option<String> {
        entry.and_then(|e| {
            if e.expire_at > Instant::now() {
                Some(e.value)
            } else {
                None
            }
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        Ok(Self::live(self.map.get(key).await))
    }

    pub async fn set_ex(&self, key: &str, val: &str, ttl_secs: u64) -> Result<(), CacheError> {
        let entry = Entry {
            value:     val.to_owned(),
            expire_at: Instant::now() + Duration::from_secs(ttl_secs),
        };
        self.map.insert(key.to_owned(), entry).await;
        Ok(())
    }

    pub async fn del(&self, key: &str) -> Result<(), CacheError> {
        self.map.invalidate(key).await;
        Ok(())
    }

    pub async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError> {
        let val = Self::live(self.map.get(key).await);
        if val.is_some() {
            self.map.invalidate(key).await;
        }
        Ok(val)
    }

    pub async fn ttl(&self, key: &str) -> Result<Option<i64>, CacheError> {
        Ok(self.map.get(key).await.and_then(|e| {
            let now = Instant::now();
            if e.expire_at > now {
                Some(e.expire_at.duration_since(now).as_secs() as i64)
            } else {
                None
            }
        }))
    }

    async fn lock_keys(&self, keys: &[String]) -> Vec<tokio::sync::MutexGuard<'_, ()>> {
        let mut idx: Vec<usize> = keys.iter().map(|k| Self::stripe_index(k)).collect();
        idx.sort_unstable();
        idx.dedup();
        let mut guards = Vec::with_capacity(idx.len());
        for i in idx {
            guards.push(self.stripes[i].lock().await);
        }
        guards
    }

    pub async fn apply_writes(&self, writes: Vec<CacheWrite>) -> Result<(), CacheError> {
        for w in writes {
            match w {
                CacheWrite::SetEx { key, val, ttl_secs } => {
                    self.set_ex(&key, &val, ttl_secs).await?;
                },
                CacheWrite::Del { key } => {
                    self.del(&key).await?;
                },
            }
        }
        Ok(())
    }

    pub async fn lock_for_tx(&self, keys: &[String]) -> Vec<tokio::sync::MutexGuard<'_, ()>> {
        self.lock_keys(keys).await
    }
}

impl TxReader for MokaCache {
    fn get<'a>(
        &'a self,
        key: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<String>, CacheError>> + Send + 'a>,
    > {
        Box::pin(async move { MokaCache::get(self, key).await })
    }

    fn ttl<'a>(
        &'a self,
        key: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<i64>, CacheError>> + Send + 'a>,
    > {
        Box::pin(async move { MokaCache::ttl(self, key).await })
    }
}
