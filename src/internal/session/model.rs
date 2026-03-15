use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

/// Default session lifetime in seconds (7 days).
pub const SESSION_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Session payload stored in Redis.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Session {
    /// User id associated with this session.
    pub uid: i32,
    /// Expiration time as unix timestamp (seconds).
    pub exp: usize,
}

/// Generate a new session payload for a given user id.
///
/// # Arguments
/// - `uid`: User id to be attached to the session.
///
/// # Returns
/// - A [`Session`] whose expiration is `now + SESSION_TTL_SECONDS`.
pub fn generate(uid: i32) -> Session {
    Session {
        uid,
        exp: (Utc::now() + Duration::seconds(SESSION_TTL_SECONDS as i64)).timestamp() as usize,
    }
}

impl Session {
    /// Check whether the session is still valid.
    ///
    /// # Returns
    /// - `true` if current time is before `exp`.
    /// - `false` if the session is expired.
    pub fn validate(&self) -> bool {
        self.exp > (Utc::now().timestamp() as usize)
    }
}

/// Parse a session from JSON text.
///
/// # Arguments
/// - `ctx`: JSON string that should represent a [`Session`].
///
/// # Returns
/// - `Some(Session)` when parsing succeeds.
/// - `None` when payload format is invalid.
pub fn parse_from_str(ctx: &str) -> Option<Session> {
    serde_json::from_str(ctx).ok()
}
