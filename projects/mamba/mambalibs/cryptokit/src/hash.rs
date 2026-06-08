//! Hashing and HMAC functions.
//!
//! Supports SHA-256, SHA-512, BLAKE3, MD5, SHA-1, and HMAC variants.

use digest::Digest;
use hmac::{Hmac, Mac};

// ── SHA-256 ──────────────────────────────────────────────

pub fn sha256(data: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(data).to_vec()
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

// ── SHA-512 ──────────────────────────────────────────────

pub fn sha512(data: &[u8]) -> Vec<u8> {
    sha2::Sha512::digest(data).to_vec()
}

pub fn sha512_hex(data: &[u8]) -> String {
    hex::encode(sha512(data))
}

// ── SHA-1 (legacy, not for security) ─────────────────────

pub fn sha1(data: &[u8]) -> Vec<u8> {
    sha1::Sha1::digest(data).to_vec()
}

pub fn sha1_hex(data: &[u8]) -> String {
    hex::encode(sha1(data))
}

// ── MD5 (legacy, not for security) ───────────────────────

pub fn md5(data: &[u8]) -> Vec<u8> {
    md5::Md5::digest(data).to_vec()
}

pub fn md5_hex(data: &[u8]) -> String {
    hex::encode(md5(data))
}

// ── BLAKE3 ───────────────────────────────────────────────

pub fn blake3_hash(data: &[u8]) -> Vec<u8> {
    blake3::hash(data).as_bytes().to_vec()
}

pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

// ── HMAC-SHA256 ──────────────────────────────────────────

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac =
        Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC-SHA256 accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    hex::encode(hmac_sha256(key, data))
}

pub fn hmac_sha256_verify(key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    let mut mac =
        Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC-SHA256 accepts any key length");
    mac.update(data);
    mac.verify_slice(signature).is_ok()
}

// ── HMAC-SHA512 ──────────────────────────────────────────

pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac =
        Hmac::<sha2::Sha512>::new_from_slice(key).expect("HMAC-SHA512 accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

pub fn hmac_sha512_hex(key: &[u8], data: &[u8]) -> String {
    hex::encode(hmac_sha512(key, data))
}

// ── HMAC-SHA1 (used by TOTP/HOTP) ───────────────────────

pub fn hmac_sha1(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac =
        Hmac::<sha1::Sha1>::new_from_slice(key).expect("HMAC-SHA1 accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256_hex(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hmac_sha256_verify() {
        let key = b"secret";
        let data = b"message";
        let sig = hmac_sha256(key, data);
        assert!(hmac_sha256_verify(key, data, &sig));
        assert!(!hmac_sha256_verify(key, b"other", &sig));
    }

    #[test]
    fn test_blake3() {
        let hash = blake3_hex(b"hello");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_md5() {
        let hash = md5_hex(b"hello");
        assert_eq!(hash, "5d41402abc4b2a76b9719d911017c592");
    }
}
