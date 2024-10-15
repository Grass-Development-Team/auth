use sha2::Digest;

pub fn generate(password: String, salt: String) -> String {
    todo!()
}

pub fn check(password: String, salt: String, hash_to_check: String) -> bool {
    let password_with_salt = format!("{}{}", password, salt);
    let hash = sha2::Sha256::digest(password_with_salt);
    let hash = base16ct::lower::encode_string(&hash);
    hash == hash_to_check
}