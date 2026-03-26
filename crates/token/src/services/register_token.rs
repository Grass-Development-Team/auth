use std::sync::OnceLock;

use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};

use crate::{TokenError, TokenStore};

const ISSUE_OR_REUSE_SCRIPT: &str = r#"
local index_key = KEYS[1]
local token_prefix = ARGV[1]
local uid = tonumber(ARGV[2])
local email = ARGV[3]
local ttl_secs = tonumber(ARGV[4])
local min_reuse_ttl_secs = tonumber(ARGV[5])
local new_token = ARGV[6]

local existing = redis.call('GET', index_key)
if existing then
    local existing_key = token_prefix .. '::' .. existing
    local payload = redis.call('GET', existing_key)
    if payload then
        local ok, decoded = pcall(cjson.decode, payload)
        if ok and decoded and tonumber(decoded.uid) == uid and decoded.email == email then
            local ttl = redis.call('TTL', existing_key)
            if ttl > min_reuse_ttl_secs then
                local index_ttl = redis.call('TTL', index_key)
                if index_ttl < ttl then
                    redis.call('EXPIRE', index_key, ttl)
                end
                return {existing, ttl}
            end
        end
    end
    redis.call('DEL', existing_key)
end

local payload = cjson.encode({ uid = uid, email = email })
local new_key = token_prefix .. '::' .. new_token
redis.call('SETEX', new_key, ttl_secs, payload)
redis.call('SETEX', index_key, ttl_secs, new_token)
return {new_token, ttl_secs}
"#;

const CONSUME_AND_CLEAR_INDEX_SCRIPT: &str = r#"
local token_key = KEYS[1]
local index_prefix = ARGV[1]
local token = ARGV[2]

local payload = redis.call('GETDEL', token_key)
if not payload then
    return nil
end

local ok, decoded = pcall(cjson.decode, payload)
if ok and decoded and decoded.uid ~= nil then
    local index_key = index_prefix .. '::' .. tostring(decoded.uid)
    local indexed_token = redis.call('GET', index_key)
    if indexed_token == token then
        redis.call('DEL', index_key)
    end
end

return payload
"#;

fn issue_or_reuse_script() -> &'static redis::Script {
    static SCRIPT: OnceLock<redis::Script> = OnceLock::new();
    SCRIPT.get_or_init(|| redis::Script::new(ISSUE_OR_REUSE_SCRIPT))
}

fn consume_and_clear_index_script() -> &'static redis::Script {
    static SCRIPT: OnceLock<redis::Script> = OnceLock::new();
    SCRIPT.get_or_init(|| redis::Script::new(CONSUME_AND_CLEAR_INDEX_SCRIPT))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterToken {
    pub uid:   i32,
    pub email: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterTokenService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterTokenLease {
    pub token:    String,
    pub ttl_secs: u64,
}

#[async_trait::async_trait]
impl TokenStore for RegisterTokenService {
    type Payload = RegisterToken;

    const PREFIX: &'static str = "register-token";
}

impl RegisterTokenService {
    const INDEX_PREFIX: &'static str = "register-token-index";

    fn index_key(uid: i32) -> String {
        format!("{}::{uid}", Self::INDEX_PREFIX)
    }

    fn token_key(token: &str) -> String {
        format!("{}::{token}", Self::PREFIX)
    }

    pub async fn issue_for_user(
        redis: &mut MultiplexedConnection,
        uid: i32,
        email: impl Into<String>,
        ttl_secs: u64,
    ) -> Result<String, TokenError> {
        let token = RegisterToken {
            uid,
            email: email.into(),
        };
        <Self as TokenStore>::issue(redis, &token, ttl_secs).await
    }

    pub async fn issue_or_reuse_for_user(
        redis: &mut MultiplexedConnection,
        uid: i32,
        email: impl Into<String>,
        ttl_secs: u64,
        min_reuse_ttl_secs: u64,
    ) -> Result<RegisterTokenLease, TokenError> {
        let email = email.into();
        let new_token = uuid::Uuid::new_v4().to_string();
        let index_key = Self::index_key(uid);
        let mut invocation = issue_or_reuse_script().prepare_invoke();
        invocation
            .key(index_key)
            .arg(Self::PREFIX)
            .arg(uid)
            .arg(email)
            .arg(ttl_secs)
            .arg(min_reuse_ttl_secs)
            .arg(new_token);
        let (token, ttl_secs): (String, i64) = invocation.invoke_async(redis).await?;

        Ok(RegisterTokenLease {
            token,
            ttl_secs: ttl_secs.max(0) as u64,
        })
    }

    pub async fn consume(
        redis: &mut MultiplexedConnection,
        token: &str,
    ) -> Result<Option<RegisterToken>, TokenError> {
        let mut invocation = consume_and_clear_index_script().prepare_invoke();
        invocation
            .key(Self::token_key(token))
            .arg(Self::INDEX_PREFIX)
            .arg(token);
        let payload: Option<String> = invocation.invoke_async(redis).await?;

        payload
            .map(|payload| serde_json::from_str(&payload))
            .transpose()
            .map_err(Into::into)
    }
}
