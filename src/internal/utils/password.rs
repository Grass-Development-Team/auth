use sha2::Digest;

pub fn generate(password: String, salt: String) -> String {
    let password_with_salt = format!("{password}{salt}");
    let hash = sha2::Sha256::digest(password_with_salt);
    base16ct::lower::encode_string(&hash)
}

pub fn check(password: String, salt: String, hash_to_check: String) -> bool {
    let hash = generate(password, salt);
    hash == hash_to_check
}
