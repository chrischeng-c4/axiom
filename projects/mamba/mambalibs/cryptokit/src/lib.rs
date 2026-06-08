//! # cclab-crypto
//!
//! Cryptography and authentication primitives.
//!
//! ## Modules
//! - `hash` — SHA-256, SHA-512, BLAKE3, MD5, SHA-1, HMAC
//! - `aead` — AES-256-GCM, ChaCha20-Poly1305
//! - `password` — bcrypt, Argon2 password hashing
//! - `jwt` — JWT encode/decode/verify
//! - `otp` — TOTP/HOTP one-time passwords
//! - `kdf` — PBKDF2, HKDF key derivation
//! - `encoding` — Base64, hex, random bytes

#[cfg(feature = "hash")]
pub mod hash;

#[cfg(feature = "aead")]
pub mod aead;

#[cfg(feature = "password")]
pub mod password;

#[cfg(feature = "jwt")]
pub mod jwt;

#[cfg(feature = "otp")]
pub mod otp;

#[cfg(feature = "kdf")]
pub mod kdf;

#[cfg(feature = "encoding")]
pub mod encoding;

#[cfg(feature = "qr")]
pub mod qr;
#[cfg(feature = "qr")]
pub mod qr_render;


/// Crypto error type.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("invalid nonce length: expected {expected}, got {got}")]
    InvalidNonceLength { expected: usize, got: usize },

    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed: {0}")]
    Decryption(String),

    #[error("password hashing failed: {0}")]
    Password(String),

    #[error("JWT error: {0}")]
    Jwt(String),

    #[error("KDF error: {0}")]
    Kdf(String),

    #[error("encoding error: {0}")]
    Encoding(String),
}
