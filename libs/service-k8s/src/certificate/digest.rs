// HANDWRITE-BEGIN gap="missing-generator:logic:certificate-digest" tracker="#3110" reason="Own the one hashing convention the lifecycle compares state by, so a fingerprint written by the projector and a fingerprint read back by the state machine cannot disagree about encoding."
//! The one hashing convention this lifecycle compares state by.
//!
//! Fingerprints are load-bearing: the state machine decides whether the leaf on
//! disk is the leaf the runtime activated by comparing them, and a mismatch in
//! case or separator would read as "not activated yet" forever. One function,
//! one encoding — lowercase hex, no colons.

use sha2::{Digest, Sha256};

/// Lowercase hex sha256 of `bytes`.
pub fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_canonical_sha256_of_the_empty_string() {
        assert_eq!(
            hex_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn is_lowercase_hex_without_separators() {
        let digest = hex_sha256(b"lumen");
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
// HANDWRITE-END
