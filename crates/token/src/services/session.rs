use chrono::{Duration, Utc};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore, backend::RedisTokenBackend};

const SESSION_PREFIX: &str = "madoka::auth::session";
const USER_SESSION_INDEX_PREFIX: &str = "madoka::auth::session-user";
const CREATE_SESSION_SCRIPT: &str = r#"
local session_key = KEYS[1]
local index_key = KEYS[2]
local payload = ARGV[1]
local ttl_secs = tonumber(ARGV[2])
local sid = ARGV[3]
local exp = tonumber(ARGV[4])
local now = tonumber(ARGV[5])

redis.call('ZADD', index_key, exp, sid)
redis.call('ZREMRANGEBYSCORE', index_key, '-inf', now)

if redis.call('ZCARD', index_key) == 0 then
    redis.call('DEL', index_key)
else
    local current_ttl = redis.call('TTL', index_key)
    if current_ttl == -1 or (current_ttl >= 0 and current_ttl < ttl_secs) then
        redis.call('EXPIRE', index_key, ttl_secs)
    end
end

redis.call('SETEX', session_key, ttl_secs, payload)

return 1
"#;
const DELETE_SESSION_SCRIPT: &str = r#"
local session_key = KEYS[1]
local sid = ARGV[1]
local index_prefix = ARGV[2]

local payload = redis.call('GETDEL', session_key)
if not payload then
    return 0
end

local ok, decoded = pcall(cjson.decode, payload)
if ok and decoded and decoded.uid ~= nil then
    local index_key = index_prefix .. '::' .. tostring(decoded.uid)
    redis.call('ZREM', index_key, sid)
end

return 1
"#;
const DELETE_ALL_BY_UID_SCRIPT: &str = r#"
local index_key = KEYS[1]
local session_prefix = ARGV[1]
local now = tonumber(ARGV[2])

redis.call('ZREMRANGEBYSCORE', index_key, '-inf', now)
local sids = redis.call('ZRANGE', index_key, 0, -1)

for _, sid in ipairs(sids) do
    local session_key = session_prefix .. '::' .. sid
    redis.call('DEL', session_key)
end

redis.call('DEL', index_key)
return #sids
"#;

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

impl SessionService {
    pub async fn create(
        redis: &mut MultiplexedConnection,
        uid: i32,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let session = generate_session(uid, ttl_secs);
        let session_id = uuid::Uuid::new_v4().to_string();
        let session_payload = serde_json::to_string(&session)?;
        let session_key = format!("{SESSION_PREFIX}::{session_id}");
        let index_key = format!("{USER_SESSION_INDEX_PREFIX}::{uid}");
        let now = Utc::now().timestamp() as usize;

        let script = redis::Script::new(CREATE_SESSION_SCRIPT);
        let mut invocation = script.prepare_invoke();
        invocation
            .key(session_key)
            .key(index_key)
            .arg(session_payload)
            .arg(ttl_secs)
            .arg(&session_id)
            .arg(session.exp)
            .arg(now);
        let _: i32 = invocation.invoke_async(redis).await?;

        Ok(session_id)
    }

    pub async fn delete(
        redis: &mut MultiplexedConnection,
        session_id: &str,
    ) -> Result<(), TokenError> {
        let session_key = format!("{SESSION_PREFIX}::{session_id}");
        let script = redis::Script::new(DELETE_SESSION_SCRIPT);
        let mut invocation = script.prepare_invoke();
        invocation
            .key(session_key)
            .arg(session_id)
            .arg(USER_SESSION_INDEX_PREFIX);
        let _: i32 = invocation.invoke_async(redis).await?;

        Ok(())
    }

    pub async fn delete_all_by_uid(
        redis: &mut MultiplexedConnection,
        uid: i32,
    ) -> Result<(), TokenError> {
        let index_key = format!("{USER_SESSION_INDEX_PREFIX}::{uid}");
        let now = Utc::now().timestamp() as usize;

        let script = redis::Script::new(DELETE_ALL_BY_UID_SCRIPT);
        let mut invocation = script.prepare_invoke();
        invocation.key(index_key).arg(SESSION_PREFIX).arg(now);
        let _: i32 = invocation.invoke_async(redis).await?;
        Ok(())
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
