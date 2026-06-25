//! Scoped claim-check access tokens (#445).
//!
//! loom's schema layer **signs** a token scoped to one task's keep keys; keep
//! **verifies** it — so a worker can GET/PUT keep directly (bytes never traverse
//! loom) but only within its scope, and only until it expires. HMAC-SHA256 over a
//! base64url(JSON) payload; both sides share a secret (out of band). Kept in one
//! crate so the signer and verifier can never drift.

use base64::Engine;
use serde::{Deserialize, Serialize};

/// What a token authorizes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    /// Readable input key (claim-check GET /v1/inputs/{r}).
    pub r: String,
    /// Writable result key (claim-check PUT /v1/results/{w}).
    pub w: String,
    /// Expiry, unix seconds.
    pub exp: u64,
}

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let ipad: Vec<u8> = block.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = block.iter().map(|b| b ^ 0x5c).collect();
    let inner = Sha256::digest([&ipad[..], msg].concat());
    Sha256::digest([&opad[..], &inner[..]].concat()).into()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Constant-time string compare (avoid signature-timing leaks).
fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Sign a scope into a token: `b64url(json) "." hex(hmac)`.
pub fn sign(secret: &[u8], scope: &Scope) -> String {
    let payload = B64.encode(serde_json::to_vec(scope).expect("encode scope"));
    let sig = hmac_sha256(secret, payload.as_bytes());
    format!("{payload}.{}", hex(&sig))
}

/// Verify a token's signature (constant-time) and expiry (`now`, unix secs);
/// return its [`Scope`] if valid.
pub fn verify(secret: &[u8], token: &str, now: u64) -> Option<Scope> {
    let (payload, sig) = token.split_once('.')?;
    if !ct_eq(&hex(&hmac_sha256(secret, payload.as_bytes())), sig) {
        return None;
    }
    let scope: Scope = serde_json::from_slice(&B64.decode(payload).ok()?).ok()?;
    (scope.exp >= now).then_some(scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> Scope {
        Scope { r: "run:a:in".into(), w: "run:a:result".into(), exp: 1000 }
    }

    #[test]
    fn sign_then_verify_roundtrips() {
        let t = sign(b"secret", &scope());
        assert_eq!(verify(b"secret", &t, 999), Some(scope()));
    }

    #[test]
    fn rejects_tamper_wrong_key_and_expiry() {
        let t = sign(b"secret", &scope());
        assert!(verify(b"WRONG", &t, 999).is_none(), "wrong secret");
        assert!(verify(b"secret", &t, 1001).is_none(), "expired (exp=1000, now=1001)");
        let tampered = format!("{}.deadbeef", t.split_once('.').unwrap().0);
        assert!(verify(b"secret", &tampered, 999).is_none(), "tampered signature");
        assert!(verify(b"secret", "no-dot", 999).is_none(), "malformed");
    }
}
