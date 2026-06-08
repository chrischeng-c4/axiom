//! Authentication validators for push receiver
//!
//! Provides OIDC JWT validation (Cloud Scheduler) and HMAC-SHA256 signature
//! validation (K8s CronJob pods).

use std::sync::Arc;
use std::time::{Duration, Instant};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::RwLock;

use crate::TaskError;

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// OIDC Validator
// ---------------------------------------------------------------------------

/// Validates OIDC JWT tokens from Google Cloud Scheduler.
///
/// Fetches Google JWKS public keys and caches them with a configurable TTL.
/// Verifies RS256 signature, `iss`, `aud`, and `exp` claims.
pub struct OidcValidator {
    /// Expected `aud` claim in the JWT
    audience: String,
    /// Expected `iss` claim (default: `https://accounts.google.com`)
    issuer: String,
    /// Google JWKS endpoint URL
    jwks_url: String,
    /// Cached JWKS public keys with TTL
    jwks_cache: Arc<RwLock<JwksCache>>,
    /// HTTP client for fetching JWKS
    http_client: reqwest::Client,
}

/// Cached Google JWKS public keys with TTL-based refresh.
pub struct JwksCache {
    /// Parsed RSA public keys from Google JWKS
    keys: Vec<jsonwebtoken::DecodingKey>,
    /// Timestamp when keys were last fetched
    fetched_at: Option<Instant>,
    /// Cache TTL
    ttl: Duration,
}

impl JwksCache {
    fn new(ttl: Duration) -> Self {
        Self {
            keys: Vec::new(),
            fetched_at: None,
            ttl,
        }
    }

    fn is_valid(&self) -> bool {
        match self.fetched_at {
            Some(fetched_at) => fetched_at.elapsed() < self.ttl,
            None => false,
        }
    }
}

impl OidcValidator {
    /// Create a new OIDC validator.
    ///
    /// # Arguments
    /// * `audience` - Expected `aud` claim in the JWT
    /// * `issuer` - Expected `iss` claim
    /// * `jwks_url` - URL to fetch Google JWKS public keys
    /// * `cache_ttl` - TTL for cached JWKS keys
    pub fn new(
        audience: String,
        issuer: String,
        jwks_url: String,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            audience,
            issuer,
            jwks_url,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new(cache_ttl))),
            http_client: reqwest::Client::new(),
        }
    }

    /// Validate a JWT token against Google JWKS.
    ///
    /// Verifies RS256 signature, issuer, audience, and expiry.
    pub async fn validate_token(&self, token: &str) -> Result<(), TaskError> {
        let keys = self.get_keys().await?;

        let mut validation =
            jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        // Try each key until one succeeds (key rotation support)
        let mut last_err = None;
        for key in &keys {
            match jsonwebtoken::decode::<serde_json::Value>(token, key, &validation) {
                Ok(_) => return Ok(()),
                Err(e) => last_err = Some(e),
            }
        }

        Err(TaskError::Authentication(format!(
            "OIDC token validation failed: {}",
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "no keys available".to_string())
        )))
    }

    /// Fetch and parse JWKS from the Google endpoint.
    pub async fn fetch_jwks(
        &self,
    ) -> Result<Vec<jsonwebtoken::DecodingKey>, TaskError> {
        let response = self
            .http_client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| {
                TaskError::Connection(format!("Failed to fetch JWKS: {}", e))
            })?;

        let jwks: jsonwebtoken::jwk::JwkSet =
            response.json().await.map_err(|e| {
                TaskError::Deserialization(format!("Failed to parse JWKS: {}", e))
            })?;

        let keys: Vec<jsonwebtoken::DecodingKey> = jwks
            .keys
            .iter()
            .filter_map(|jwk| jsonwebtoken::DecodingKey::from_jwk(jwk).ok())
            .collect();

        if keys.is_empty() {
            return Err(TaskError::Configuration(
                "No valid keys in JWKS response".to_string(),
            ));
        }

        Ok(keys)
    }

    /// Check if the cached JWKS keys are still within their TTL.
    pub fn is_cache_valid(&self) -> bool {
        // Non-async check — uses try_read to avoid blocking.
        // For authoritative checks, use get_keys() which acquires the lock.
        self.jwks_cache
            .try_read()
            .map(|cache| cache.is_valid())
            .unwrap_or(false)
    }

    /// Get JWKS keys, fetching from the endpoint if cache is expired.
    async fn get_keys(
        &self,
    ) -> Result<Vec<jsonwebtoken::DecodingKey>, TaskError> {
        // Check cache
        {
            let cache = self.jwks_cache.read().await;
            if cache.is_valid() {
                return Ok(cache.keys.clone());
            }
        }

        // Cache expired or empty — fetch new keys
        let keys = self.fetch_jwks().await?;

        {
            let mut cache = self.jwks_cache.write().await;
            cache.keys = keys.clone();
            cache.fetched_at = Some(Instant::now());
        }

        Ok(keys)
    }
}

// ---------------------------------------------------------------------------
// HMAC Validator
// ---------------------------------------------------------------------------

/// Validates HMAC-SHA256 signatures from K8s CronJob pods.
///
/// The expected header format is `X-Scheduler-Signature: sha256={hex_digest}`.
/// Uses constant-time comparison via the `hmac` crate's `verify_slice`.
#[derive(Debug)]
pub struct HmacValidator {
    /// Raw HMAC secret bytes
    secret: Vec<u8>,
}

impl HmacValidator {
    /// Create a new HMAC validator.
    ///
    /// The secret must be at least 32 bytes.
    pub fn new(secret: &[u8]) -> Result<Self, TaskError> {
        if secret.len() < 32 {
            return Err(TaskError::Configuration(
                "HMAC secret must be at least 32 bytes".to_string(),
            ));
        }
        Ok(Self {
            secret: secret.to_vec(),
        })
    }

    /// Validate the HMAC-SHA256 signature from the `X-Scheduler-Signature` header.
    ///
    /// Expected format: `sha256={hex_digest}`
    ///
    /// Uses constant-time comparison internally via `hmac::Mac::verify_slice`.
    pub fn validate_signature(
        &self,
        body: &[u8],
        signature_header: &str,
    ) -> Result<(), TaskError> {
        let hex_digest =
            signature_header.strip_prefix("sha256=").ok_or_else(|| {
                TaskError::Authentication(
                    "HMAC signature validation failed".to_string(),
                )
            })?;

        let provided_bytes = hex_decode(hex_digest).map_err(|_| {
            TaskError::Authentication(
                "HMAC signature validation failed".to_string(),
            )
        })?;

        let mut mac = HmacSha256::new_from_slice(&self.secret).map_err(|e| {
            TaskError::Configuration(format!("HMAC key error: {}", e))
        })?;
        mac.update(body);

        // Constant-time comparison
        mac.verify_slice(&provided_bytes).map_err(|_| {
            TaskError::Authentication(
                "HMAC signature validation failed".to_string(),
            )
        })
    }

    /// Compute the HMAC-SHA256 signature for a body.
    ///
    /// Returns `sha256={hex_digest}`.
    pub fn compute_signature(&self, body: &[u8]) -> String {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret).expect("valid HMAC key");
        mac.update(body);
        let result = mac.finalize();
        let bytes = result.into_bytes();
        format!("sha256={}", hex_encode(&bytes))
    }
}

// ---------------------------------------------------------------------------
// Hex utilities (avoid external dep)
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Hex utilities
    // -----------------------------------------------------------------------

    #[test]
    fn hex_encode_roundtrip() {
        let data = b"hello world";
        let encoded = hex_encode(data);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(b""), "");
    }

    #[test]
    fn hex_decode_odd_length_fails() {
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn hex_decode_invalid_chars_fails() {
        assert!(hex_decode("zzzz").is_err());
    }

    // -----------------------------------------------------------------------
    // JwksCache
    // -----------------------------------------------------------------------

    #[test]
    fn jwks_cache_new_is_invalid() {
        let cache = JwksCache::new(Duration::from_secs(3600));
        assert!(!cache.is_valid(), "Fresh JwksCache should be invalid (no keys fetched)");
        assert!(cache.keys.is_empty());
        assert!(cache.fetched_at.is_none());
    }

    #[test]
    fn jwks_cache_valid_within_ttl() {
        let mut cache = JwksCache::new(Duration::from_secs(3600));
        cache.fetched_at = Some(Instant::now());
        // Just fetched, well within TTL
        assert!(cache.is_valid());
    }

    #[test]
    fn jwks_cache_invalid_after_ttl() {
        let mut cache = JwksCache::new(Duration::from_millis(1));
        cache.fetched_at = Some(Instant::now() - Duration::from_millis(50));
        // TTL of 1ms has long passed
        assert!(!cache.is_valid());
    }

    // -----------------------------------------------------------------------
    // OidcValidator construction
    // -----------------------------------------------------------------------

    #[test]
    fn oidc_validator_construction() {
        let validator = OidcValidator::new(
            "https://app.example.com".to_string(),
            "https://accounts.google.com".to_string(),
            "https://www.googleapis.com/oauth2/v3/certs".to_string(),
            Duration::from_secs(3600),
        );
        assert_eq!(validator.audience, "https://app.example.com");
        assert_eq!(validator.issuer, "https://accounts.google.com");
        assert_eq!(validator.jwks_url, "https://www.googleapis.com/oauth2/v3/certs");
    }

    #[test]
    fn oidc_validator_cache_initially_invalid() {
        let validator = OidcValidator::new(
            "aud".to_string(),
            "iss".to_string(),
            "url".to_string(),
            Duration::from_secs(3600),
        );
        assert!(!validator.is_cache_valid());
    }

    // -----------------------------------------------------------------------
    // HmacValidator — S2, S4 from spec
    // -----------------------------------------------------------------------

    fn make_hmac_validator() -> HmacValidator {
        // 32-byte secret (minimum required)
        let secret = b"this-is-a-32-byte-hmac-secret!!x";
        assert!(secret.len() >= 32);
        HmacValidator::new(secret).unwrap()
    }

    #[test]
    fn hmac_validator_new_with_valid_secret() {
        let result = HmacValidator::new(b"this-is-a-32-byte-hmac-secret!!x");
        assert!(result.is_ok());
    }

    #[test]
    fn hmac_validator_new_with_short_secret_fails() {
        // Secret less than 32 bytes must fail (spec constraint)
        let result = HmacValidator::new(b"too-short");
        assert!(result.is_err());
        match result.unwrap_err() {
            TaskError::Configuration(msg) => {
                assert!(msg.contains("at least 32 bytes"), "Error: {msg}");
            }
            other => panic!("Expected Configuration error, got: {:?}", other),
        }
    }

    #[test]
    fn hmac_validator_new_with_exactly_32_bytes() {
        let secret = b"abcdefghijklmnopqrstuvwxyz012345"; // exactly 32 bytes
        assert_eq!(secret.len(), 32);
        assert!(HmacValidator::new(secret).is_ok());
    }

    #[test]
    fn hmac_validator_new_with_31_bytes_fails() {
        let secret = b"abcdefghijklmnopqrstuvwxyz01234"; // 31 bytes
        assert_eq!(secret.len(), 31);
        assert!(HmacValidator::new(secret).is_err());
    }

    // S2: K8s CronJob triggers push receiver with valid HMAC signature
    #[test]
    fn s2_hmac_compute_and_validate_roundtrip() {
        let validator = make_hmac_validator();
        let body = b"test request body";

        let signature = validator.compute_signature(body);
        assert!(signature.starts_with("sha256="), "Signature should have sha256= prefix");

        let result = validator.validate_signature(body, &signature);
        assert!(result.is_ok(), "Roundtrip should succeed");
    }

    #[test]
    fn s2_hmac_signature_format() {
        let validator = make_hmac_validator();
        let body = b"hello";

        let signature = validator.compute_signature(body);
        assert!(signature.starts_with("sha256="));
        // hex part should be 64 chars (SHA-256 = 32 bytes = 64 hex chars)
        let hex_part = signature.strip_prefix("sha256=").unwrap();
        assert_eq!(hex_part.len(), 64, "SHA-256 hex digest should be 64 chars");
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // S4: Request with invalid HMAC signature is rejected
    #[test]
    fn s4_hmac_validate_wrong_signature() {
        let validator = make_hmac_validator();
        let body = b"test body";

        // Compute signature for different body
        let wrong_sig = validator.compute_signature(b"different body");

        let result = validator.validate_signature(body, &wrong_sig);
        assert!(result.is_err(), "Wrong signature should fail");
        match result.unwrap_err() {
            TaskError::Authentication(msg) => {
                assert!(msg.contains("HMAC signature validation failed"), "Error: {msg}");
            }
            other => panic!("Expected Authentication error, got: {:?}", other),
        }
    }

    #[test]
    fn s4_hmac_validate_missing_prefix() {
        let validator = make_hmac_validator();
        let body = b"test";

        // No sha256= prefix
        let result = validator.validate_signature(body, "abcdef1234567890");
        assert!(result.is_err());
    }

    #[test]
    fn s4_hmac_validate_invalid_hex() {
        let validator = make_hmac_validator();
        let body = b"test";

        let result = validator.validate_signature(body, "sha256=not-valid-hex!");
        assert!(result.is_err());
    }

    #[test]
    fn s4_hmac_validate_empty_signature_header() {
        let validator = make_hmac_validator();
        let body = b"test";

        let result = validator.validate_signature(body, "");
        assert!(result.is_err());
    }

    #[test]
    fn s4_hmac_validate_correct_prefix_but_wrong_digest() {
        let validator = make_hmac_validator();
        let body = b"test body";

        // Correct format but completely wrong digest
        let fake_sig = "sha256=0000000000000000000000000000000000000000000000000000000000000000";
        let result = validator.validate_signature(body, fake_sig);
        assert!(result.is_err());
    }

    // S2: Different body contents produce different signatures
    #[test]
    fn s2_hmac_different_bodies_different_signatures() {
        let validator = make_hmac_validator();
        let sig1 = validator.compute_signature(b"body one");
        let sig2 = validator.compute_signature(b"body two");
        assert_ne!(sig1, sig2, "Different bodies should produce different signatures");
    }

    // S2: Same body always produces same signature (deterministic)
    #[test]
    fn s2_hmac_deterministic() {
        let validator = make_hmac_validator();
        let body = b"deterministic test";
        let sig1 = validator.compute_signature(body);
        let sig2 = validator.compute_signature(body);
        assert_eq!(sig1, sig2, "Same body should always produce same signature");
    }

    // S2: Empty body is valid
    #[test]
    fn s2_hmac_empty_body() {
        let validator = make_hmac_validator();
        let body = b"";
        let sig = validator.compute_signature(body);
        assert!(validator.validate_signature(body, &sig).is_ok());
    }

    // S4: Signature computed with different secret fails
    #[test]
    fn s4_hmac_different_secret_fails() {
        let secret1 = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 32 bytes
        let secret2 = b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"; // 32 bytes
        let v1 = HmacValidator::new(secret1).unwrap();
        let v2 = HmacValidator::new(secret2).unwrap();

        let body = b"shared body";
        let sig_from_v1 = v1.compute_signature(body);

        // Validate with different-secret validator
        let result = v2.validate_signature(body, &sig_from_v1);
        assert!(result.is_err(), "Signature from different secret should fail");
    }
}
