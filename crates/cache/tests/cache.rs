use std::sync::Arc;

use cache::{Cache, MokaCache, RedisCache};

fn moka() -> Cache {
    Cache::Moka(MokaCache::new(10_000))
}

fn redis() -> Cache {
    Cache::Redis(RedisCache::new("redis://127.0.0.1:6379").unwrap())
}

#[tokio::test]
async fn primitives_set_get_del() {
    let c = moka();
    assert_eq!(c.get("k").await.unwrap(), None);
    c.set_ex("k", "v", 60).await.unwrap();
    assert_eq!(c.get("k").await.unwrap(), Some("v".into()));
    assert_eq!(c.get_del("k").await.unwrap(), Some("v".into()));
    assert_eq!(c.get("k").await.unwrap(), None);
}

#[tokio::test]
async fn ttl_reports_remaining() {
    let c = moka();
    c.set_ex("k", "v", 60).await.unwrap();
    let ttl = c.ttl("k").await.unwrap().unwrap();
    assert!(ttl > 0 && ttl <= 60);
    assert_eq!(c.ttl("missing").await.unwrap(), None);
}

#[tokio::test]
async fn transaction_reads_and_buffers_writes() {
    let c = moka();
    c.set_ex("idx", "old", 60).await.unwrap();
    let out: i32 = c
        .transaction(&["idx".to_string()], |tx| {
            Box::pin(async move {
                let cur = tx.get("idx").await?;
                assert_eq!(cur.as_deref(), Some("old"));
                tx.set_ex("idx", "new", 60);
                tx.set_ex("extra", "1", 60);
                Ok(7)
            })
        })
        .await
        .unwrap();
    assert_eq!(out, 7);
    assert_eq!(c.get("idx").await.unwrap(), Some("new".into()));
    assert_eq!(c.get("extra").await.unwrap(), Some("1".into()));
}

#[tokio::test]
async fn transaction_concurrent_atomicity() {
    let c = Arc::new(moka());
    c.set_ex("counter", "0", 60).await.unwrap();

    let mut handles = Vec::new();
    for _ in 0..50 {
        let c = c.clone();
        handles.push(tokio::spawn(async move {
            c.transaction(&["counter".to_string()], |tx| {
                Box::pin(async move {
                    let cur: i64 = tx.get("counter").await?.unwrap().parse().unwrap();
                    tx.set_ex("counter", (cur + 1).to_string(), 60);
                    Ok(())
                })
            })
            .await
            .unwrap();
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(c.get("counter").await.unwrap(), Some("50".into()));
}

#[tokio::test]
#[ignore = "requires a running redis"]
async fn redis_primitives() {
    let c = redis();
    let key = format!("cache-test::{}", uuid_like());
    c.set_ex(&key, "v", 60).await.unwrap();
    assert_eq!(c.get(&key).await.unwrap(), Some("v".into()));
    assert_eq!(c.get_del(&key).await.unwrap(), Some("v".into()));
    assert_eq!(c.get(&key).await.unwrap(), None);
}

#[tokio::test]
#[ignore = "requires a running redis"]
async fn redis_transaction() {
    let c = redis();
    let key = format!("cache-test-tx::{}", uuid_like());
    c.set_ex(&key, "0", 60).await.unwrap();
    let out: i64 = c
        .transaction(&[key.clone()], |tx| {
            let key = key.clone();
            Box::pin(async move {
                let cur: i64 = tx.get(&key).await?.unwrap().parse().unwrap();
                tx.set_ex(&key, (cur + 1).to_string(), 60);
                Ok(cur + 1)
            })
        })
        .await
        .unwrap();
    assert_eq!(out, 1);
    assert_eq!(c.get(&key).await.unwrap(), Some("1".into()));
}

fn uuid_like() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
