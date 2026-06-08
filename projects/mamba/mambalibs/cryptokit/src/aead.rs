//! Authenticated encryption (AEAD).
//!
//! Supports AES-256-GCM and ChaCha20-Poly1305.

use crate::CryptoError;
use aes_gcm::aead::{Aead, KeyInit, Payload};
use rand::RngCore;

const KEY_LEN: usize = 32; // 256-bit key
const NONCE_LEN: usize = 12; // 96-bit nonce

/// Generate a random 256-bit key.
pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Generate a random 96-bit nonce.
pub fn generate_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

fn validate_key_nonce(key: &[u8], nonce: &[u8]) -> Result<(), CryptoError> {
    if key.len() != KEY_LEN {
        return Err(CryptoError::InvalidKeyLength {
            expected: KEY_LEN,
            got: key.len(),
        });
    }
    if nonce.len() != NONCE_LEN {
        return Err(CryptoError::InvalidNonceLength {
            expected: NONCE_LEN,
            got: nonce.len(),
        });
    }
    Ok(())
}

// ── AES-256-GCM ─────────────────────────────────────────

pub fn aes_gcm_encrypt(
    key: &[u8],
    nonce: &[u8],
    plaintext: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    validate_key_nonce(key, nonce)?;
    let cipher = aes_gcm::Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;
    let nonce = aes_gcm::Nonce::from_slice(nonce);
    match aad {
        Some(aad) => cipher
            .encrypt(nonce, Payload { msg: plaintext, aad })
            .map_err(|e| CryptoError::Encryption(e.to_string())),
        None => cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::Encryption(e.to_string())),
    }
}

pub fn aes_gcm_decrypt(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    validate_key_nonce(key, nonce)?;
    let cipher = aes_gcm::Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::Decryption(e.to_string()))?;
    let nonce = aes_gcm::Nonce::from_slice(nonce);
    match aad {
        Some(aad) => cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|e| CryptoError::Decryption(e.to_string())),
        None => cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Decryption(e.to_string())),
    }
}

// ── ChaCha20-Poly1305 ───────────────────────────────────

pub fn chacha20_encrypt(
    key: &[u8],
    nonce: &[u8],
    plaintext: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    validate_key_nonce(key, nonce)?;
    let cipher = chacha20poly1305::ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;
    let nonce = chacha20poly1305::Nonce::from_slice(nonce);
    match aad {
        Some(aad) => cipher
            .encrypt(
                nonce,
                chacha20poly1305::aead::Payload { msg: plaintext, aad },
            )
            .map_err(|e| CryptoError::Encryption(e.to_string())),
        None => cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::Encryption(e.to_string())),
    }
}

pub fn chacha20_decrypt(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    validate_key_nonce(key, nonce)?;
    let cipher = chacha20poly1305::ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::Decryption(e.to_string()))?;
    let nonce = chacha20poly1305::Nonce::from_slice(nonce);
    match aad {
        Some(aad) => cipher
            .decrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|e| CryptoError::Decryption(e.to_string())),
        None => cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Decryption(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = generate_key();
        let nonce = generate_nonce();
        let plaintext = b"hello world";
        let ct = aes_gcm_encrypt(&key, &nonce, plaintext, None).unwrap();
        let pt = aes_gcm_decrypt(&key, &nonce, &ct, None).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_aes_gcm_with_aad() {
        let key = generate_key();
        let nonce = generate_nonce();
        let plaintext = b"secret data";
        let aad = b"metadata";
        let ct = aes_gcm_encrypt(&key, &nonce, plaintext, Some(aad)).unwrap();
        let pt = aes_gcm_decrypt(&key, &nonce, &ct, Some(aad)).unwrap();
        assert_eq!(pt, plaintext);
        // Wrong AAD should fail
        assert!(aes_gcm_decrypt(&key, &nonce, &ct, Some(b"wrong")).is_err());
    }

    #[test]
    fn test_chacha20_roundtrip() {
        let key = generate_key();
        let nonce = generate_nonce();
        let plaintext = b"hello chacha";
        let ct = chacha20_encrypt(&key, &nonce, plaintext, None).unwrap();
        let pt = chacha20_decrypt(&key, &nonce, &ct, None).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_invalid_key_length() {
        let nonce = generate_nonce();
        let err = aes_gcm_encrypt(b"short", &nonce, b"data", None).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKeyLength { .. }));
    }
}
