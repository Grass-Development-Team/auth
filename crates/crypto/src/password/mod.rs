mod algorithm;
mod salt;

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
}

pub struct PasswordManager;

impl PasswordManager {
    pub fn generate_salt() -> String {
        salt::generate(24)
    }

    pub fn hash_password(
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

    pub fn verify_password(password: &str, encoded: &str) -> Result<bool, PasswordError> {
        let (hash_type, content) = encoded
            .split_once(":")
            .ok_or(PasswordError::InvalidHashFormat)?;

        match hash_type {
            "sha2" => algorithm::sha2::verify_password(password, content),
            "argon2id" => algorithm::argon2id::verify_password(password, content),
            other => Err(PasswordError::UnsupportedHashType(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PasswordHashAlgorithm, PasswordManager};

    #[test]
    fn hash_password_sha2_uses_type_content_format() {
        let salt = PasswordManager::generate_salt();
        let encoded =
            PasswordManager::hash_password(PasswordHashAlgorithm::Sha256, "passw0rd", &salt)
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
        let encoded =
            PasswordManager::hash_password(PasswordHashAlgorithm::Argon2id, "passw0rd", &salt)
                .expect("argon2id hash should succeed");

        assert!(encoded.starts_with("argon2id:$argon2id$"));
    }

    #[test]
    fn verify_password_works_for_sha2_and_argon2id() {
        let sha2 =
            PasswordManager::hash_password(PasswordHashAlgorithm::Sha256, "passw0rd", "salt123")
                .expect("sha2 hash should succeed");
        assert!(
            PasswordManager::verify_password("passw0rd", &sha2)
                .expect("sha2 verify should succeed")
        );
        assert!(
            !PasswordManager::verify_password("wrong", &sha2).expect("sha2 verify should succeed")
        );

        let salt = PasswordManager::generate_salt();
        let argon2id =
            PasswordManager::hash_password(PasswordHashAlgorithm::Argon2id, "passw0rd", &salt)
                .expect("argon2id hash should succeed");
        assert!(
            PasswordManager::verify_password("passw0rd", &argon2id)
                .expect("argon2id verify should succeed")
        );
        assert!(
            !PasswordManager::verify_password("wrong", &argon2id)
                .expect("argon2id verify should succeed")
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
}
