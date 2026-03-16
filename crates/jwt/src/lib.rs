use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Represents the claims contained within a JWT token.
#[derive(Deserialize, Serialize, Debug)]
pub struct Claim<T = ()> {
    /// Issuer
    pub iss: String,
    /// Username
    pub sub: String,
    /// The time of expiration
    pub exp: usize,
    /// User ID
    pub uid: i32,
    /// Data associated with the claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dat: Option<T>,
}

/// Encodes a [`Claim`] into a JWT string using the provided secret.
///
/// # Arguments
///
/// * `claim` - The [`Claim`] to encode.
/// * `secret` - The secret key used to sign the JWT.
///
/// # Returns
///
/// A [`jsonwebtoken::errors::Result<String>`] containing the encoded JWT
/// string, or an error if encoding fails.
pub fn generate<T: Serialize>(
    claim: Claim<T>,
    secret: &str,
) -> jsonwebtoken::errors::Result<String> {
    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/// Creates a new [`Claim`] with a 7-day expiration from the current time.
///
/// # Arguments
///
/// * `iss` - The issuer of the token.
/// * `sub` - The subject (username) of the token.
/// * `uid` - The user ID associated with the token.
/// * `dat` - Optional custom payload attached to the claim.
///
/// # Returns
///
/// A [`Claim`] populated with the provided values and an expiration set to 7
/// days from now.
pub fn generate_claim<T>(iss: String, sub: String, uid: i32, dat: Option<T>) -> Claim<T> {
    Claim {
        iss,
        sub,
        exp: (Utc::now() + Duration::days(7)).timestamp() as usize,
        uid,
        dat,
    }
}

/// Decodes and validates a JWT string, returning the contained [`Claim`].
///
/// # Arguments
///
/// * `jwt` - The JWT string to decode.
/// * `secret` - The secret key used to verify the JWT signature.
///
/// # Returns
///
/// A [`jsonwebtoken::errors::Result<Claim<T>>`] containing the decoded
/// [`Claim<T>`],
/// or an error if decoding or validation fails.
pub fn unwrap<T: DeserializeOwned>(
    jwt: &str,
    secret: &str,
) -> jsonwebtoken::errors::Result<Claim<T>> {
    let claim = decode::<Claim<T>>(
        jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(claim.claims)
}

/// Checks whether a [`Claim`] has not yet expired.
///
/// # Arguments
///
/// * `claim` - A reference to the [`Claim`] to validate.
///
/// # Returns
///
/// `true` if the claim's expiration time is in the future, `false` otherwise.
pub fn validate<T>(claim: &Claim<T>) -> bool {
    claim.exp > (Utc::now().timestamp() as usize)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use jsonwebtoken::errors::ErrorKind;
    use serde::{Deserialize, Serialize};

    use super::{Claim, generate, generate_claim, unwrap, validate};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Extra {
        sid: String,
    }

    fn build_claim(exp: usize) -> Claim<Extra> {
        Claim {
            iss: "gdt-auth".to_owned(),
            sub: "leo".to_owned(),
            exp,
            uid: 42,
            dat: Some(Extra {
                sid: "session::123".to_owned(),
            }),
        }
    }

    #[test]
    fn generate_and_unwrap_round_trip_with_same_secret() {
        let secret = "top-secret";
        let exp = (Utc::now() + Duration::minutes(30)).timestamp() as usize;
        let claim = build_claim(exp);

        let token = generate(claim, secret).expect("token generation should succeed");
        let decoded = unwrap::<Extra>(&token, secret).expect("token decode should succeed");

        assert_eq!(decoded.iss, "gdt-auth");
        assert_eq!(decoded.sub, "leo");
        assert_eq!(decoded.uid, 42);
        assert_eq!(
            decoded.dat.as_ref().map(|v| v.sid.as_str()),
            Some("session::123")
        );
        assert_eq!(decoded.exp, exp);
    }

    #[test]
    fn generate_and_unwrap_round_trip_without_dat() {
        let secret = "top-secret";
        let exp = (Utc::now() + Duration::minutes(30)).timestamp() as usize;
        let claim = Claim::<()> {
            iss: "gdt-auth".to_owned(),
            sub: "leo".to_owned(),
            exp,
            uid: 42,
            dat: None,
        };

        let token = generate(claim, secret).expect("token generation should succeed");
        let decoded = unwrap::<()>(&token, secret).expect("token decode should succeed");

        assert_eq!(decoded.iss, "gdt-auth");
        assert_eq!(decoded.sub, "leo");
        assert_eq!(decoded.uid, 42);
        assert!(
            decoded.dat.is_none(),
            "dat should remain None after round-trip"
        );
        assert_eq!(decoded.exp, exp);
    }

    #[test]
    fn unwrap_fails_with_wrong_secret_and_invalid_signature_error() {
        let exp = (Utc::now() + Duration::minutes(30)).timestamp() as usize;
        let claim = build_claim(exp);
        let token = generate(claim, "secret::a").expect("token generation should succeed");

        let err = unwrap::<Extra>(&token, "secret::b")
            .expect_err("decoding with wrong secret should fail");
        assert!(
            matches!(err.kind(), ErrorKind::InvalidSignature),
            "expected InvalidSignature, got {:?}",
            err.kind()
        );
    }

    #[test]
    fn unwrap_fails_for_expired_token_with_expired_signature_error() {
        let exp = (Utc::now() - Duration::minutes(2)).timestamp() as usize;
        let claim = build_claim(exp);
        let token = generate(claim, "secret").expect("token generation should succeed");

        let err = unwrap::<Extra>(&token, "secret").expect_err("expired token should fail");
        assert!(
            matches!(err.kind(), ErrorKind::ExpiredSignature),
            "expected ExpiredSignature, got {:?}",
            err.kind()
        );
    }

    #[test]
    fn validate_checks_expiration_boundary() {
        let now = Utc::now().timestamp() as usize;
        let past = build_claim(now.saturating_sub(1));
        let equal_now = build_claim(now);
        let future = build_claim(now + 60);

        assert!(!validate(&past));
        assert!(!validate(&equal_now));
        assert!(validate(&future));
    }

    #[test]
    fn generate_claim_sets_exp_to_about_seven_days() {
        let before = Utc::now().timestamp() as usize;
        let claim = generate_claim(
            "gdt-auth".to_owned(),
            "leo".to_owned(),
            42,
            Some(Extra {
                sid: "session::123".to_owned(),
            }),
        );
        let after = Utc::now().timestamp() as usize;

        let lower = before + Duration::days(7).num_seconds() as usize;
        let upper = after + Duration::days(7).num_seconds() as usize;
        assert!(
            claim.exp >= lower && claim.exp <= upper,
            "claim.exp should be within [now_before + 7d, now_after + 7d], got {}, range: [{}, {}]",
            claim.exp,
            lower,
            upper
        );
        assert_eq!(
            claim.dat.as_ref().map(|v| v.sid.as_str()),
            Some("session::123")
        );
    }
}
