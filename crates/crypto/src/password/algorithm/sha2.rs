use sha2::Digest;
use subtle::ConstantTimeEq;

use crate::password::PasswordError;

pub fn hash_password(password: &str, salt: &str) -> String {
    let password_with_salt = format!("{password}{salt}");
    let hash = sha2::Sha256::digest(password_with_salt);
    let hash = base16ct::lower::encode_string(&hash);

    format!("{hash}:{salt}")
}

pub fn verify_password(password: &str, content: &str) -> Result<bool, PasswordError> {
    let (expected_hash, salt) = content
        .split_once(":")
        .ok_or(PasswordError::InvalidHashFormat)?;

    let actual = hash_password(password, salt);
    let (actual_hash, _) = actual
        .split_once(":")
        .ok_or(PasswordError::InvalidHashFormat)?;

    Ok(bool::from(
        actual_hash.as_bytes().ct_eq(expected_hash.as_bytes()),
    ))
}
