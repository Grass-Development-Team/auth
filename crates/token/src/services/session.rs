use std::collections::HashMap;

use cache::Cache;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore, backend::CacheTokenBackend};

const SESSION_PREFIX: &str = "madoka::auth::session";
const USER_SESSION_INDEX_PREFIX: &str = "madoka::auth::session-user";

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Session {
    pub uid: i32,
    pub exp: usize,
}

pub fn generate_session(uid: i32, ttl_secs: u64) -> Session {
    Session {
        uid,
        exp: (Utc::now() + Duration::seconds(ttl_secs as i64)).timestamp() as usize,
    }
}

impl Session {
    pub fn validate(&self) -> bool {
        self.exp > (Utc::now().timestamp() as usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SessionLookup {
    Missing,
    Invalid,
    Expired,
    Valid(Session),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionService;

#[async_trait::async_trait]
impl TokenStore for SessionService {
    type Payload = Session;

    const PREFIX: &'static str = SESSION_PREFIX;
}

fn session_key(sid: &str) -> String {
    format!("{SESSION_PREFIX}::{sid}")
}

fn index_key(uid: i32) -> String {
    format!("{USER_SESSION_INDEX_PREFIX}::{uid}")
}

fn parse_index(raw: Option<String>, now: usize) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> = raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    map.retain(|_, exp| *exp > now);
    map
}

impl SessionService {
    pub async fn create(cache: &Cache, uid: i32, ttl_secs: u64) -> Result<String, TokenError> {
        let session = generate_session(uid, ttl_secs);
        let session_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::to_string(&session)?;
        let idx_key = index_key(uid);
        let sess_key = session_key(&session_id);
        let sid = session_id.clone();
        let watch = [idx_key.clone()];

        cache
            .transaction(&watch, move |tx| {
                let (idx_key, sess_key, payload, sid) = (
                    idx_key.clone(),
                    sess_key.clone(),
                    payload.clone(),
                    sid.clone(),
                );
                Box::pin(async move {
                    let now = Utc::now().timestamp() as usize;
                    let mut index = parse_index(tx.get(&idx_key).await?, now);
                    index.insert(sid.clone(), session.exp);
                    let index_ttl = index.values().copied().max().unwrap_or(0);
                    let index_ttl = (index_ttl as i64 - now as i64).max(0) as u64;
                    let index_json = serde_json::to_string(&index)
                        .map_err(|e| cache::CacheError::Backend(e.into()))?;
                    tx.set_ex(&idx_key, index_json, index_ttl.max(ttl_secs));
                    tx.set_ex(&sess_key, payload.clone(), ttl_secs);
                    Ok(())
                })
            })
            .await?;

        Ok(session_id)
    }

    pub async fn delete(cache: &Cache, session_id: &str) -> Result<(), TokenError> {
        let sess_key = session_key(session_id);
        let Some(payload) = cache.get(&sess_key).await? else {
            return Ok(());
        };
        let session: Session = serde_json::from_str(&payload)?;
        let idx_key = index_key(session.uid);
        let sid = session_id.to_owned();
        let watch = [idx_key.clone()];

        cache
            .transaction(&watch, move |tx| {
                let (idx_key, sess_key, sid) = (idx_key.clone(), sess_key.clone(), sid.clone());
                Box::pin(async move {
                    let now = Utc::now().timestamp() as usize;
                    let mut index = parse_index(tx.get(&idx_key).await?, now);
                    index.remove(&sid);
                    if index.is_empty() {
                        tx.del(&idx_key);
                    } else {
                        let json = serde_json::to_string(&index)
                            .map_err(|e| cache::CacheError::Backend(e.into()))?;
                        let ttl = (index.values().copied().max().unwrap_or(now) as i64 - now as i64)
                            .max(1) as u64;
                        tx.set_ex(&idx_key, json, ttl);
                    }
                    tx.del(&sess_key);
                    Ok(())
                })
            })
            .await?;
        Ok(())
    }

    pub async fn delete_all_by_uid(cache: &Cache, uid: i32) -> Result<(), TokenError> {
        let idx_key = index_key(uid);
        let watch = [idx_key.clone()];
        cache
            .transaction(&watch, move |tx| {
                let idx_key = idx_key.clone();
                Box::pin(async move {
                    let index = parse_index(tx.get(&idx_key).await?, 0);
                    for sid in index.keys() {
                        tx.del(session_key(sid));
                    }
                    tx.del(&idx_key);
                    Ok(())
                })
            })
            .await?;
        Ok(())
    }

    pub async fn resolve(cache: &Cache, session_id: &str) -> Result<SessionLookup, TokenError> {
        let payload = CacheTokenBackend::get_raw(cache, Self::PREFIX, session_id).await?;
        let Some(payload) = payload else {
            return Ok(SessionLookup::Missing);
        };

        let session = match serde_json::from_str::<Session>(&payload) {
            Ok(session) => session,
            Err(_) => return Ok(SessionLookup::Invalid),
        };

        if !session.validate() {
            return Ok(SessionLookup::Expired);
        }

        Ok(SessionLookup::Valid(session))
    }
}
