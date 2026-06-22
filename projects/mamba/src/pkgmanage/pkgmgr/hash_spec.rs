// hash_spec.rs — `--hash=algo:hex` flag parser.
//
// pip-compile and uv pin write hash annotations on every line in their
// lockfile-style requirements.txt output:
//
//     requests==2.31.0 \
//         --hash=sha256:58cd2187c01e70e6e26505bca751777aa9f2ee0b7f4300988b709f44aaaa…
//         --hash=sha256:942c5a758f98d790eaed1a29cb6eef6cdd47cb2dabd3a9b1f87bd97c12e6…
//
// Each `--hash=algo:hex` annotation declares one acceptable digest for
// the file pip will download. Multiple annotations are OR-ed (any one
// match accepts the file). requirements_parse already preserves these
// as opaque strings on `PackageRequirement.hashes`; this module is the
// typed-validation layer on top — convert that string list into
// `Vec<FileHash>` and reject anything that can't be a real hash.
//
// Algorithms recognised: sha256 (canonical), sha384, sha512, sha3_256,
// sha3_384, sha3_512, blake2b, blake2s, md5 (deprecated but still
// present in the wild). The digest must be lowercase hex of the right
// length for its algorithm — uppercase hex is accepted on parse and
// normalized to lowercase on the way out (matches pip's behavior).

use crate::pkgmanage::pkgmgr::types::{FileHash, IndexError};

/// Parse one `algo:hex` token (no leading `--hash=`).
pub fn parse_hash_spec(spec: &str) -> Result<FileHash, IndexError> {
    let s = spec.trim();
    if s.is_empty() {
        return Err(pe("empty hash spec"));
    }
    let (algo, hex) = match s.split_once(':') {
        Some(x) => x,
        None => {
            return Err(pe(&format!("hash spec must be 'algo:hex', got {s:?}")));
        }
    };
    let algo = algo.trim();
    let hex = hex.trim();
    let expected_len = match algo.to_ascii_lowercase().as_str() {
        "md5" => 32,
        "sha1" => 40,
        "sha224" => 56,
        "sha256" => 64,
        "sha384" => 96,
        "sha512" => 128,
        "sha3_224" => 56,
        "sha3_256" => 64,
        "sha3_384" => 96,
        "sha3_512" => 128,
        "blake2s" => 64,
        "blake2b" => 128,
        other => {
            return Err(pe(&format!("unknown hash algorithm {other:?}")));
        }
    };
    if hex.len() != expected_len {
        return Err(pe(&format!(
            "hash digest for {algo} must be {expected_len} hex chars, got {}",
            hex.len()
        )));
    }
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(pe(&format!("hash digest is not hex: {hex:?}")));
    }
    Ok(FileHash {
        algorithm: algo.to_ascii_lowercase(),
        digest: hex.to_ascii_lowercase(),
    })
}

/// Parse a `--hash=algo:hex` flag with its leading prefix.
pub fn parse_hash_flag(flag: &str) -> Result<FileHash, IndexError> {
    let s = flag.trim();
    let rest = s
        .strip_prefix("--hash=")
        .ok_or_else(|| pe(&format!("expected '--hash=' prefix, got {s:?}")))?;
    parse_hash_spec(rest)
}

/// Validate a list of opaque hash strings into typed FileHash values.
/// Returns the first error encountered; collects all successes when
/// `Ok`.
pub fn parse_hash_specs(specs: &[String]) -> Result<Vec<FileHash>, IndexError> {
    specs.iter().map(|s| parse_hash_spec(s)).collect()
}

fn pe(msg: &str) -> IndexError {
    IndexError::ParseError {
        url: "--hash".into(),
        detail: msg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fh(algo: &str, hex: &str) -> FileHash {
        FileHash {
            algorithm: algo.into(),
            digest: hex.into(),
        }
    }

    const SHA256_X: &str = "9d4ca56a6dc5b2c5cfc4c9bb7f8b69bdc25ad7a4b8e8fb6e6f5e0a3a1b2c3d4e";
    const SHA512_X: &str = concat!(
        "9d4ca56a6dc5b2c5cfc4c9bb7f8b69bdc25ad7a4b8e8fb6e6f5e0a3a1b2c3d4e",
        "9d4ca56a6dc5b2c5cfc4c9bb7f8b69bdc25ad7a4b8e8fb6e6f5e0a3a1b2c3d4e",
    );

    #[test]
    fn parse_sha256_spec() {
        let h = parse_hash_spec(&format!("sha256:{SHA256_X}")).unwrap();
        assert_eq!(h, fh("sha256", SHA256_X));
    }

    #[test]
    fn parse_sha512_spec() {
        let h = parse_hash_spec(&format!("sha512:{SHA512_X}")).unwrap();
        assert_eq!(h.algorithm, "sha512");
        assert_eq!(h.digest.len(), 128);
    }

    #[test]
    fn parse_hash_flag_with_prefix() {
        let h = parse_hash_flag(&format!("--hash=sha256:{SHA256_X}")).unwrap();
        assert_eq!(h.algorithm, "sha256");
    }

    #[test]
    fn rejects_missing_prefix_on_flag_form() {
        let err = parse_hash_flag(&format!("sha256:{SHA256_X}")).unwrap_err();
        assert!(err.to_string().contains("'--hash=' prefix"));
    }

    #[test]
    fn algorithm_case_normalized_to_lowercase() {
        let h = parse_hash_spec(&format!("SHA256:{SHA256_X}")).unwrap();
        assert_eq!(h.algorithm, "sha256");
    }

    #[test]
    fn digest_case_normalized_to_lowercase() {
        let upper = SHA256_X.to_ascii_uppercase();
        let h = parse_hash_spec(&format!("sha256:{upper}")).unwrap();
        assert_eq!(h.digest, SHA256_X);
    }

    #[test]
    fn rejects_unknown_algorithm() {
        let err = parse_hash_spec(&format!("crc32:{SHA256_X}")).unwrap_err();
        assert!(err.to_string().contains("unknown hash algorithm"));
    }

    #[test]
    fn rejects_missing_colon() {
        let err = parse_hash_spec("sha256-deadbeef").unwrap_err();
        assert!(err.to_string().contains("algo:hex"));
    }

    #[test]
    fn rejects_empty_string() {
        let err = parse_hash_spec("").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn rejects_wrong_digest_length() {
        // sha256 wants 64 hex chars — feed it sha512's 128.
        let err = parse_hash_spec(&format!("sha256:{SHA512_X}")).unwrap_err();
        assert!(err.to_string().contains("64 hex chars"));
    }

    #[test]
    fn rejects_non_hex_digest() {
        let bad = "z".repeat(64);
        let err = parse_hash_spec(&format!("sha256:{bad}")).unwrap_err();
        assert!(err.to_string().contains("not hex"));
    }

    #[test]
    fn md5_supported_for_legacy_pip_compile() {
        let md5 = "d41d8cd98f00b204e9800998ecf8427e";
        let h = parse_hash_spec(&format!("md5:{md5}")).unwrap();
        assert_eq!(h.algorithm, "md5");
        assert_eq!(h.digest.len(), 32);
    }

    #[test]
    fn sha1_supported() {
        let sha1 = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
        let h = parse_hash_spec(&format!("sha1:{sha1}")).unwrap();
        assert_eq!(h.algorithm, "sha1");
    }

    #[test]
    fn parse_specs_collects_in_order() {
        let specs = vec![format!("sha256:{SHA256_X}"), format!("sha512:{SHA512_X}")];
        let hashes = parse_hash_specs(&specs).unwrap();
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0].algorithm, "sha256");
        assert_eq!(hashes[1].algorithm, "sha512");
    }

    #[test]
    fn parse_specs_returns_first_error() {
        let specs = vec![
            format!("sha256:{SHA256_X}"),
            "not-a-hash".to_string(),
            format!("sha256:{SHA256_X}"),
        ];
        let err = parse_hash_specs(&specs).unwrap_err();
        assert!(err.to_string().contains("algo:hex"));
    }

    #[test]
    fn parse_specs_empty_list_yields_empty_vec() {
        let v: Vec<FileHash> = parse_hash_specs(&[]).unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn whitespace_around_colon_tolerated() {
        let h = parse_hash_spec(&format!("sha256 : {SHA256_X}")).unwrap();
        assert_eq!(h.algorithm, "sha256");
    }

    #[test]
    fn whitespace_around_flag_tolerated() {
        let h = parse_hash_flag(&format!("   --hash=sha256:{SHA256_X}   ")).unwrap();
        assert_eq!(h.algorithm, "sha256");
    }
}
