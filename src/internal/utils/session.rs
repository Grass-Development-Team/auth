use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

pub const SESSION_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub uid: i32,
    pub exp: usize,
}

pub fn generate(uid: i32) -> Session {
    Session {
        uid,
        exp: (Utc::now() + Duration::seconds(SESSION_TTL_SECONDS as i64)).timestamp() as usize,
    }
}

impl Session {
    pub fn validate(&self) -> bool {
        self.exp > (Utc::now().timestamp() as usize)
    }
}

pub fn parse_from_str(ctx: &str) -> Option<Session> {
    serde_json::from_str(ctx).ok()
}
