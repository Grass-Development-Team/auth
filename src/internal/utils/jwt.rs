use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
    pub uid: u32,
    /// Session ID
    /// This is to manage jwt through server.
    pub sid: String,
}

pub fn generate(claim: Claim, secret: &str) -> jsonwebtoken::errors::Result<String> {
    encode(&Header::default(), &claim, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn unwrap(jwt: &str, secret: &str) -> jsonwebtoken::errors::Result<Claim> {
    let claim = match decode::<Claim>(jwt, &DecodingKey::from_secret(secret.as_ref()), &Validation::default()) {
        Ok(claim) => claim,
        Err(err) => return Err(err),
    };
    Ok(claim.claims)
}