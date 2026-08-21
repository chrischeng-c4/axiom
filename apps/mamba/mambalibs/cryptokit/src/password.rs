//! Password hashing with bcrypt and Argon2.

use crate::CryptoError;

// ── bcrypt ───────────────────────────────────────────────

/// Hash a password with bcrypt. Default cost is 12.
pub fn bcrypt_hash(password: &str, cost: Option<u32>) -> Result<String, CryptoError> {
    let cost = cost.unwrap_or(bcrypt::DEFAULT_COST);
    bcrypt::hash(password, cost).map_err(|e| CryptoError::Password(e.to_string()))
}

/// Verify a password against a bcrypt hash.
pub fn bcrypt_verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
    bcrypt::verify(password, hash).map_err(|e| CryptoError::Password(e.to_string()))
}

// ── Argon2 ───────────────────────────────────────────────

/// Hash a password with Argon2id (recommended for password storage).
pub fn argon2_hash(password: &str) -> Result<String, CryptoError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::Password(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a password against an Argon2 hash string.
pub fn argon2_verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
    let parsed = PasswordHash::new(hash).map_err(|e| CryptoError::Password(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcrypt_roundtrip() {
        let hash = bcrypt_hash("mypassword", Some(4)).unwrap(); // cost=4 for fast tests
        assert!(bcrypt_verify("mypassword", &hash).unwrap());
        assert!(!bcrypt_verify("wrong", &hash).unwrap());
    }

    #[test]
    fn test_argon2_roundtrip() {
        let hash = argon2_hash("securepass").unwrap();
        assert!(hash.starts_with("$argon2"));
        assert!(argon2_verify("securepass", &hash).unwrap());
        assert!(!argon2_verify("wrong", &hash).unwrap());
    }
}
