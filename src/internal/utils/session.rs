use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub uid: i32,
    pub exp: usize,
}

pub fn generate(uid: i32) -> Session {
    Session {
        uid,
        exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
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
