use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};

use crate::password::PasswordError;

pub fn hash_password(password: &str, salt: &str) -> Result<String, PasswordError> {
    let salt = SaltString::from_b64(salt).map_err(|_| PasswordError::InvalidSaltEncoding)?;

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| PasswordError::Argon2(err.to_string()))
}

pub fn verify_password(password: &str, content: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(content).map_err(|_| PasswordError::InvalidHashFormat)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(err) => Err(PasswordError::Argon2(err.to_string())),
    }
}
