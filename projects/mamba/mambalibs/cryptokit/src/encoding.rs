//! Encoding utilities: base64, hex, and random byte generation.

use crate::CryptoError;
use base64::Engine;
use rand::RngCore;

// ── Base64 ───────────────────────────────────────────────

pub fn base64_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>, CryptoError> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| CryptoError::Encoding(e.to_string()))
}

pub fn base64url_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn base64url_decode(s: &str) -> Result<Vec<u8>, CryptoError> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|e| CryptoError::Encoding(e.to_string()))
}

// ── Hex ──────────────────────────────────────────────────

pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, CryptoError> {
    hex::decode(s).map_err(|e| CryptoError::Encoding(e.to_string()))
}

// ── Random ───────────────────────────────────────────────

/// Generate cryptographically secure random bytes.
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

/// Generate a random hex string of the given byte length.
pub fn random_hex(byte_len: usize) -> String {
    hex::encode(random_bytes(byte_len))
}

/// Generate a random string from a given charset.
pub fn random_string(len: usize, charset: Option<&str>) -> String {
    use rand::Rng;
    let charset =
        charset.unwrap_or("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
    let chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64url_roundtrip() {
        let data = b"\xff\xfe\xfd";
        let encoded = base64url_encode(data);
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = b"hello";
        let encoded = hex_encode(data);
        assert_eq!(encoded, "68656c6c6f");
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_random_bytes() {
        let a = random_bytes(32);
        let b = random_bytes(32);
        assert_eq!(a.len(), 32);
        assert_ne!(a, b); // Extremely unlikely to be equal
    }

    #[test]
    fn test_random_string() {
        let s = random_string(16, None);
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
