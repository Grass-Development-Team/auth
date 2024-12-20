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

pub fn validate(session: &Session) -> bool {
    session.exp < (Utc::now().timestamp() as usize)
}