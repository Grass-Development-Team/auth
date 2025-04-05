use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Claim {
    /// Issuer
    pub iss: String,
    /// Username
    pub sub: String,
    /// The time of expiration
    pub exp: usize,
    /// User ID
    pub uid: i32,
    /// Session ID
    /// This is to manage jwt through server.
    pub sid: String,
}

pub fn generate(claim: Claim, secret: &str) -> jsonwebtoken::errors::Result<String> {
    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn generate_claim(iss: String, sub: String, uid: i32, sid: String) -> Claim {
    Claim {
        iss,
        sub,
        exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
        uid,
        sid,
    }
}

pub fn unwrap(jwt: &str, secret: &str) -> jsonwebtoken::errors::Result<Claim> {
    let claim = decode::<Claim>(
        jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(claim.claims)
}

pub fn validate(claim: &Claim) -> bool {
    claim.exp < (Utc::now().timestamp() as usize)
}
