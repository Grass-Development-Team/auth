use chrono::{Duration, Utc};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore, backend::RedisTokenBackend};

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

    const PREFIX: &'static str = "session";
}

impl SessionService {
    pub async fn create(
        redis: &mut MultiplexedConnection,
        uid: i32,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let session = generate_session(uid, ttl_secs);
        <Self as TokenStore>::issue(redis, &session, ttl_secs).await
    }

    pub async fn delete(
        redis: &mut MultiplexedConnection,
        session_id: &str,
    ) -> Result<(), TokenError> {
        <Self as TokenStore>::revoke(redis, session_id).await
    }

    pub async fn resolve(
        redis: &mut MultiplexedConnection,
        session_id: &str,
    ) -> Result<SessionLookup, TokenError> {
        let payload = RedisTokenBackend::get_raw(redis, Self::PREFIX, session_id).await?;
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
