use base64ct::{Base64Unpadded, Encoding};
use rand::{RngCore, rngs::OsRng};

/// Minimum/maximum PHC salt character length.
const MIN_SALT_LENGTH: usize = 4;
const MAX_SALT_LENGTH: usize = 64;

/// Maximum source entropy bytes that can be encoded into a PHC salt under the
/// `MAX_SALT_LENGTH` constraint.
const MAX_SALT_BYTES: usize = 48;

/// Generate a cryptographically secure PHC-compatible salt using the given
/// random byte length.
pub fn generate(byte_len: usize) -> String {
    let byte_len = byte_len.clamp(1, MAX_SALT_BYTES);
    let mut bytes = vec![0u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    let salt = Base64Unpadded::encode_string(&bytes);
    debug_assert!((MIN_SALT_LENGTH..=MAX_SALT_LENGTH).contains(&salt.len()));
    salt
}

#[cfg(test)]
mod tests {
    use super::generate;

    fn is_phc_salt(s: &str) -> bool {
        if s.len() < 4 || s.len() > 64 {
            return false;
        }

        s.chars()
            .all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '+' | '.' | '-'))
    }

    #[test]
    fn generate_returns_phc_salt() {
        let salt = generate(16);
        assert!(!salt.is_empty());
        assert!(is_phc_salt(&salt));
    }

    #[test]
    fn generate_clamps_too_large_entropy_input() {
        let salt = generate(9999);
        assert!(is_phc_salt(&salt));
        assert!(salt.len() >= 4);
        assert!(salt.len() <= 64);
    }
}
