use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Claim {
    /// Issuer
    pub iss: &'static str,
    /// Username
    pub sub: String,
    /// The time of expiration
    pub exp: usize,
    /// User ID
    pub uid: u32,
    /// Session ID
    /// This is to manage jwt through server.
    pub sid: String,
}

#[derive(Deserialize, Serialize)]
struct Header {
    pub alg: &'static str,
    pub typ: &'static str,
}

const HEAD: Header = Header {
    alg: "HS256",
    typ: "JWT",
};

pub fn generate(claim: Claim, salt: &str) -> () {
    todo!()
}