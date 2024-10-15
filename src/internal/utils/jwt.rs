use serde::{Deserialize, Serialize};

#[serde(Deserialize, Serialize, Debug)]
pub struct Claim {
    /// Issuer
    pub iss: &str,
    /// Username
    pub sub: String,
    /// The time of expiration
    pub exp: usize,
    /// User ID
    pub uid: u32,
    /// Session ID
    /// This is to manage jwt throght server.
    pub sid: String,
}

#[serde(Deserialize, Serialize)]
struct Header {
    pub alg: &str,
    pub typ: &str,
}

pub fn generate() -> () {
    todo!()
}