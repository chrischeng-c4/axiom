//! Key derivation functions: PBKDF2 and HKDF.

use crate::CryptoError;

/// Derive a key using PBKDF2-HMAC-SHA256.
pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], rounds: u32, output_len: usize) -> Vec<u8> {
    let mut output = vec![0u8; output_len];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, rounds, &mut output);
    output
}

/// Derive a key using HKDF-SHA256.
pub fn hkdf_sha256(
    ikm: &[u8],
    salt: Option<&[u8]>,
    info: &[u8],
    output_len: usize,
) -> Result<Vec<u8>, CryptoError> {
    let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(salt, ikm);
    let mut output = vec![0u8; output_len];
    hkdf.expand(info, &mut output)
        .map_err(|e| CryptoError::Kdf(e.to_string()))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_deterministic() {
        let a = pbkdf2_sha256(b"password", b"salt", 1000, 32);
        let b = pbkdf2_sha256(b"password", b"salt", 1000, 32);
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
    }

    #[test]
    fn test_pbkdf2_different_passwords() {
        let a = pbkdf2_sha256(b"password1", b"salt", 1000, 32);
        let b = pbkdf2_sha256(b"password2", b"salt", 1000, 32);
        assert_ne!(a, b);
    }

    #[test]
    fn test_hkdf_sha256() {
        let key = hkdf_sha256(b"input key material", Some(b"salt"), b"info", 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_hkdf_no_salt() {
        let key = hkdf_sha256(b"ikm", None, b"context", 64).unwrap();
        assert_eq!(key.len(), 64);
    }
}
