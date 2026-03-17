mod algorithm;
mod salt;

const PASSWORD_MIN_LEN: usize = 8;
const PASSWORD_MAX_LEN: usize = 64;

const PASSWORD_SPECIALS: &[u8] = b"!@#$%^&*()_+-=[]{};':\",.<>/?";

pub enum PasswordHashAlgorithm {
    // Only for compatibility with existing hashes
    Sha256,
    // Recommended for new hashes
    Argon2id,
}

impl PasswordHashAlgorithm {
    fn as_type_name(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha2",
            Self::Argon2id => "argon2id",
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PasswordError {
    #[error("invalid password hash format")]
    InvalidHashFormat,
    #[error("unsupported password hash type: {0}")]
    UnsupportedHashType(String),
    #[error("invalid argon2id salt")]
    InvalidSaltEncoding,
    #[error("argon2id error: {0}")]
    Argon2(String),
    #[error("password must be between {PASSWORD_MIN_LEN} and {PASSWORD_MAX_LEN} characters")]
    InvalidPasswordLength,
    #[error(
        "password contains unsupported characters; allowed: letters, numbers, and \
         !@#$%^&*()_+-=[]{{}};':\",.<>/?"
    )]
    InvalidPasswordCharset,
    #[error("password must contain at least one letter")]
    PasswordMissingLetter,
    #[error("password must contain at least one number")]
    PasswordMissingNumber,
}

pub struct PasswordManager;

impl PasswordManager {
    pub fn generate_salt() -> String {
        salt::generate(24)
    }

    /// Hash a password using the specified algorithm and salt.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The algorithm to use for hashing.
    /// * `password` - The password to hash.
    /// * `salt` - The salt to use for hashing.
    ///
    /// # Returns
    ///
    /// A `Result` containing the hashed password, or an error if hashing fails.
    pub fn hash(
        algorithm: PasswordHashAlgorithm,
        password: &str,
        salt: &str,
    ) -> Result<String, PasswordError> {
        let content = match algorithm {
            PasswordHashAlgorithm::Sha256 => algorithm::sha2::hash_password(password, salt),
            PasswordHashAlgorithm::Argon2id => algorithm::argon2id::hash_password(password, salt)?,
        };

        Ok(format!("{}:{content}", algorithm.as_type_name()))
    }

    /// Verify a password against an encoded hash.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to verify.
    /// * `encoded` - The encoded hash to verify against.
    ///
    /// # Returns
    ///
    /// A `Result` containing `true` if the password matches the hash, or
    /// `false` otherwise, or an error if verification fails.
    pub fn verify(password: &str, encoded: &str) -> Result<bool, PasswordError> {
        let (hash_type, content) = encoded
            .split_once(":")
            .ok_or(PasswordError::InvalidHashFormat)?;

        match hash_type {
            "sha2" => algorithm::sha2::verify_password(password, content),
            "argon2id" => algorithm::argon2id::verify_password(password, content),
            other => Err(PasswordError::UnsupportedHashType(other.to_string())),
        }
    }

    /// Validate a password against the password policy.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to validate.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Ok(())` if the password is valid, or an error
    /// if it fails validation.
    pub fn validate(password: &str) -> Result<(), PasswordError> {
        if !(PASSWORD_MIN_LEN..=PASSWORD_MAX_LEN).contains(&password.len()) {
            return Err(PasswordError::InvalidPasswordLength);
        }

        if !password.chars().all(is_supported_password_char) {
            return Err(PasswordError::InvalidPasswordCharset);
        }

        if !password.chars().any(|ch| ch.is_ascii_alphabetic()) {
            return Err(PasswordError::PasswordMissingLetter);
        }

        if !password.chars().any(|ch| ch.is_ascii_digit()) {
            return Err(PasswordError::PasswordMissingNumber);
        }

        Ok(())
    }
}

#[inline]
fn is_supported_password_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || (ch.is_ascii() && PASSWORD_SPECIALS.contains(&(ch as u8)))
}

#[cfg(test)]
mod tests {
    use super::{PasswordError, PasswordHashAlgorithm, PasswordManager};

    #[test]
    fn hash_password_sha2_uses_type_content_format() {
        let salt = PasswordManager::generate_salt();
        let encoded = PasswordManager::hash(PasswordHashAlgorithm::Sha256, "passw0rd", &salt)
            .expect("sha2 hash should succeed");

        let parts = encoded.split(":").collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "sha2");
        assert_eq!(parts[2], &salt);
        assert_eq!(parts[1].len(), 64);
    }

    #[test]
    fn hash_password_argon2id_uses_type_content_format() {
        let salt = PasswordManager::generate_salt();
        let encoded = PasswordManager::hash(PasswordHashAlgorithm::Argon2id, "passw0rd", &salt)
            .expect("argon2id hash should succeed");

        assert!(encoded.starts_with("argon2id:$argon2id$"));
    }

    #[test]
    fn verify_password_works_for_sha2_and_argon2id() {
        let sha2 = PasswordManager::hash(PasswordHashAlgorithm::Sha256, "passw0rd", "salt123")
            .expect("sha2 hash should succeed");
        assert!(PasswordManager::verify("passw0rd", &sha2).expect("sha2 verify should succeed"));
        assert!(!PasswordManager::verify("wrong", &sha2).expect("sha2 verify should succeed"));

        let salt = PasswordManager::generate_salt();
        let argon2id = PasswordManager::hash(PasswordHashAlgorithm::Argon2id, "passw0rd", &salt)
            .expect("argon2id hash should succeed");
        assert!(
            PasswordManager::verify("passw0rd", &argon2id).expect("argon2id verify should succeed")
        );
        assert!(
            !PasswordManager::verify("wrong", &argon2id).expect("argon2id verify should succeed")
        );
    }

    #[test]
    fn generate_salt_produces_phc_compatible_string() {
        let salt = PasswordManager::generate_salt();

        assert!(!salt.is_empty());
        assert!((4..=64).contains(&salt.len()));
        assert!(
            salt.chars()
                .all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '+' | '.' | '-'))
        );
    }

    #[test]
    fn validate_password_accepts_strong_password() {
        let valid = "Abcdef12!@";
        assert_eq!(PasswordManager::validate(valid), Ok(()));
    }

    #[test]
    fn validate_password_rejects_invalid_passwords() {
        assert_eq!(
            PasswordManager::validate("abc"),
            Err(PasswordError::InvalidPasswordLength)
        );
        assert_eq!(
            PasswordManager::validate("Abcdef12`"),
            Err(PasswordError::InvalidPasswordCharset)
        );
        assert_eq!(
            PasswordManager::validate("12345678"),
            Err(PasswordError::PasswordMissingLetter)
        );
        assert_eq!(
            PasswordManager::validate("abcdefgh"),
            Err(PasswordError::PasswordMissingNumber)
        );
    }
}
