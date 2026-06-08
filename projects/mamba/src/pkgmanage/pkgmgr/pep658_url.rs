// pep658_url.rs — compose .metadata sidecar URLs + parse index hints.
//
// PEP 658 (clarified by PEP 714) lets an index advertise that a
// `<wheel-url>.metadata` sidecar is available so clients can fetch the
// METADATA file without downloading the wheel itself. The hint can
// arrive in two shapes:
//
//   * HTML simple API (PEP 503 with PEP 714 attribute):
//       <a href="…wheel.whl" data-core-metadata="sha256=abc…">
//       <a href="…wheel.whl" data-dist-info-metadata="sha256=abc…">
//       <a href="…wheel.whl" data-core-metadata="true">
//
//   * JSON simple API (PEP 691):
//       {
//         "filename": "x.whl", "url": "…/x.whl",
//         "core-metadata": {"sha256": "abc…"}          // hashed
//         "dist-info-metadata": true                    // unhashed
//       }
//
// This module covers three operations:
//
//   1. `compose_sidecar_url(wheel_url)` — append `.metadata` to the
//      wheel URL. Trims trailing query / fragment (mirrors uv).
//
//   2. `parse_hint_attr(attr_value)` — interpret the HTML attribute
//      string into a typed `MetadataHint`.
//
//   3. `parse_hint_json(value)` — interpret the JSON field value
//      (bool or object) into the same `MetadataHint`.
//
// When `MetadataHint::Absent`, the caller should fall back to the
// PEP 658 alternative path: fetch the wheel's central-directory tail
// via `range.rs` and extract METADATA from the zip stream.

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::{FileHash, IndexError};

/// Whether the index advertised a `.metadata` sidecar — and, if so,
/// whether it pinned a digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataHint {
    /// No sidecar advertised. Caller must use the zip-tail fallback.
    Absent,
    /// Sidecar is available but the index did not declare a digest.
    /// Clients must trust the bytes received (or compute their own
    /// digest from the wheel after the fact).
    AvailableUnhashed,
    /// Sidecar with declared digest. Client should verify the fetched
    /// bytes against this hash before parsing METADATA.
    AvailableHashed(FileHash),
}

impl MetadataHint {
    /// Convenience: did the index promise a sidecar (with or without digest)?
    pub fn is_available(&self) -> bool {
        !matches!(self, MetadataHint::Absent)
    }
}

/// Append `.metadata` to a wheel URL, dropping any query string or
/// fragment first. Caller is responsible for ensuring the URL points
/// at a `.whl` file — passing in a non-wheel URL is allowed but the
/// resulting sidecar URL will not resolve.
pub fn compose_sidecar_url(wheel_url: &str) -> String {
    let trimmed_q = wheel_url
        .find('?')
        .map(|i| &wheel_url[..i])
        .unwrap_or(wheel_url);
    let trimmed = trimmed_q
        .find('#')
        .map(|i| &trimmed_q[..i])
        .unwrap_or(trimmed_q);
    format!("{trimmed}.metadata")
}

/// Parse the HTML simple-API attribute value (the contents of
/// `data-core-metadata="…"` or `data-dist-info-metadata="…"`).
///
/// Accepted shapes:
///   * `""`            — attribute present but empty → `AvailableUnhashed`
///     (matches uv's tolerance for the HTML-attribute corner case).
///   * `"true"`        — `AvailableUnhashed`.
///   * `"false"`       — `Absent`.
///   * `"<algo>=<hex>"`— `AvailableHashed`.
pub fn parse_hint_attr(value: &str) -> Result<MetadataHint, IndexError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(MetadataHint::AvailableUnhashed);
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "true" => return Ok(MetadataHint::AvailableUnhashed),
        "false" => return Ok(MetadataHint::Absent),
        _ => {}
    }
    let (algo, digest) = trimmed
        .split_once('=')
        .ok_or_else(|| IndexError::ParseError {
            url: String::new(),
            detail: format!("PEP 658 hint attribute is not '<algo>=<hex>': {trimmed:?}"),
        })?;
    validate_algo_digest(algo, digest)?;
    Ok(MetadataHint::AvailableHashed(FileHash {
        algorithm: algo.to_ascii_lowercase(),
        digest: digest.to_ascii_lowercase(),
    }))
}

/// Parse the JSON simple-API field value. Accepts:
///   * absent / `null`         → `Absent`
///   * `true`                  → `AvailableUnhashed`
///   * `false`                 → `Absent`
///   * `{"sha256": "abc…"}`    → `AvailableHashed` (first entry; PEP
///     714 says clients SHOULD pick a single strong digest).
pub fn parse_hint_json(value: Option<&serde_json::Value>) -> Result<MetadataHint, IndexError> {
    let v = match value {
        None | Some(serde_json::Value::Null) => return Ok(MetadataHint::Absent),
        Some(v) => v,
    };
    if let Some(b) = v.as_bool() {
        return Ok(if b {
            MetadataHint::AvailableUnhashed
        } else {
            MetadataHint::Absent
        });
    }
    let obj = v.as_object().ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: format!("PEP 658 JSON hint must be bool or object, got {v}"),
    })?;
    if obj.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "PEP 658 JSON hint object is empty".into(),
        });
    }
    let (algo, digest_v) = obj.iter().next().expect("non-empty checked above");
    let digest = digest_v.as_str().ok_or_else(|| IndexError::ParseError {
        url: String::new(),
        detail: format!("PEP 658 JSON hint digest must be a string for '{algo}'"),
    })?;
    validate_algo_digest(algo, digest)?;
    Ok(MetadataHint::AvailableHashed(FileHash {
        algorithm: algo.to_ascii_lowercase(),
        digest: digest.to_ascii_lowercase(),
    }))
}

fn validate_algo_digest(algo: &str, digest: &str) -> Result<(), IndexError> {
    if algo.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "PEP 658 hint has empty algorithm".into(),
        });
    }
    if digest.is_empty() {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: "PEP 658 hint has empty digest".into(),
        });
    }
    if !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!("PEP 658 hint digest is not hex: {digest:?}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- compose_sidecar_url ------------------------------------------

    #[test]
    fn compose_basic_wheel_url() {
        assert_eq!(
            compose_sidecar_url("https://pypi.org/packages/mypkg-1.0-py3-none-any.whl"),
            "https://pypi.org/packages/mypkg-1.0-py3-none-any.whl.metadata"
        );
    }

    #[test]
    fn compose_strips_query_string() {
        assert_eq!(
            compose_sidecar_url("https://cdn/mypkg.whl?token=abc&exp=123"),
            "https://cdn/mypkg.whl.metadata"
        );
    }

    #[test]
    fn compose_strips_fragment() {
        assert_eq!(
            compose_sidecar_url("https://cdn/mypkg.whl#sha256=xyz"),
            "https://cdn/mypkg.whl.metadata"
        );
    }

    #[test]
    fn compose_strips_both_query_and_fragment() {
        assert_eq!(
            compose_sidecar_url("https://cdn/mypkg.whl?token=abc#hash"),
            "https://cdn/mypkg.whl.metadata"
        );
    }

    #[test]
    fn compose_works_on_relative_url() {
        // PEP 503 indexes ship relative hrefs; we still append `.metadata`.
        assert_eq!(
            compose_sidecar_url("./mypkg-1.0-py3-none-any.whl"),
            "./mypkg-1.0-py3-none-any.whl.metadata"
        );
    }

    // ---- parse_hint_attr ----------------------------------------------

    #[test]
    fn attr_empty_is_unhashed() {
        assert_eq!(
            parse_hint_attr("").unwrap(),
            MetadataHint::AvailableUnhashed
        );
    }

    #[test]
    fn attr_true_is_unhashed() {
        assert_eq!(
            parse_hint_attr("true").unwrap(),
            MetadataHint::AvailableUnhashed
        );
        assert_eq!(
            parse_hint_attr("TRUE").unwrap(),
            MetadataHint::AvailableUnhashed
        );
    }

    #[test]
    fn attr_false_is_absent() {
        assert_eq!(parse_hint_attr("false").unwrap(), MetadataHint::Absent);
    }

    #[test]
    fn attr_with_hash_is_hashed() {
        let hint = parse_hint_attr(
            "sha256=ABCdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        )
        .unwrap();
        match hint {
            MetadataHint::AvailableHashed(fh) => {
                assert_eq!(fh.algorithm, "sha256");
                // Digest is lowercased.
                assert!(fh.digest.starts_with("abcdef"));
                assert!(fh.digest.chars().all(|c| c.is_ascii_hexdigit()));
            }
            other => panic!("expected AvailableHashed, got {other:?}"),
        }
    }

    #[test]
    fn attr_rejects_missing_equals() {
        let err = parse_hint_attr("sha256abc").unwrap_err();
        assert!(err_detail(err).contains("'<algo>=<hex>'"));
    }

    #[test]
    fn attr_rejects_empty_algo() {
        let err = parse_hint_attr("=abcdef").unwrap_err();
        assert!(err_detail(err).contains("empty algorithm"));
    }

    #[test]
    fn attr_rejects_empty_digest() {
        let err = parse_hint_attr("sha256=").unwrap_err();
        assert!(err_detail(err).contains("empty digest"));
    }

    #[test]
    fn attr_rejects_non_hex_digest() {
        let err = parse_hint_attr("sha256=zzz").unwrap_err();
        assert!(err_detail(err).contains("not hex"));
    }

    // ---- parse_hint_json ----------------------------------------------

    #[test]
    fn json_none_is_absent() {
        assert_eq!(parse_hint_json(None).unwrap(), MetadataHint::Absent);
    }

    #[test]
    fn json_null_is_absent() {
        assert_eq!(
            parse_hint_json(Some(&serde_json::Value::Null)).unwrap(),
            MetadataHint::Absent
        );
    }

    #[test]
    fn json_true_is_unhashed() {
        let v = serde_json::Value::Bool(true);
        assert_eq!(
            parse_hint_json(Some(&v)).unwrap(),
            MetadataHint::AvailableUnhashed
        );
    }

    #[test]
    fn json_false_is_absent() {
        let v = serde_json::Value::Bool(false);
        assert_eq!(parse_hint_json(Some(&v)).unwrap(), MetadataHint::Absent);
    }

    #[test]
    fn json_object_with_sha256_is_hashed() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"}"#,
        )
        .unwrap();
        let hint = parse_hint_json(Some(&v)).unwrap();
        match hint {
            MetadataHint::AvailableHashed(fh) => {
                assert_eq!(fh.algorithm, "sha256");
                assert_eq!(fh.digest.len(), 64);
            }
            other => panic!("expected AvailableHashed, got {other:?}"),
        }
    }

    #[test]
    fn json_rejects_empty_object() {
        let v = serde_json::Value::Object(serde_json::Map::new());
        let err = parse_hint_json(Some(&v)).unwrap_err();
        assert!(err_detail(err).contains("object is empty"));
    }

    #[test]
    fn json_rejects_non_string_digest() {
        let v: serde_json::Value = serde_json::from_str(r#"{"sha256": 123}"#).unwrap();
        let err = parse_hint_json(Some(&v)).unwrap_err();
        assert!(err_detail(err).contains("must be a string"));
    }

    #[test]
    fn json_rejects_non_object_non_bool() {
        let v: serde_json::Value = serde_json::from_str(r#""yes""#).unwrap();
        let err = parse_hint_json(Some(&v)).unwrap_err();
        assert!(err_detail(err).contains("must be bool or object"));
    }

    #[test]
    fn json_rejects_non_hex_digest() {
        let v: serde_json::Value = serde_json::from_str(r#"{"sha256": "zzz"}"#).unwrap();
        let err = parse_hint_json(Some(&v)).unwrap_err();
        assert!(err_detail(err).contains("not hex"));
    }

    // ---- is_available --------------------------------------------------

    #[test]
    fn is_available_flag() {
        assert!(!MetadataHint::Absent.is_available());
        assert!(MetadataHint::AvailableUnhashed.is_available());
        assert!(MetadataHint::AvailableHashed(FileHash {
            algorithm: "sha256".into(),
            digest: "0".repeat(64),
        })
        .is_available());
    }

    // ---- realistic --------------------------------------------------

    #[test]
    fn realistic_pep691_json_release_file() {
        // Excerpt from a PEP 691 JSON simple-API "files" entry.
        let entry: serde_json::Value = serde_json::from_str(
            r#"{
                "filename": "mypkg-1.0-py3-none-any.whl",
                "url": "https://files.pythonhosted.org/.../mypkg-1.0-py3-none-any.whl",
                "core-metadata": {"sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"}
            }"#,
        )
        .unwrap();
        let hint = parse_hint_json(entry.get("core-metadata")).unwrap();
        assert!(hint.is_available());

        let wheel_url = entry["url"].as_str().unwrap();
        let sidecar = compose_sidecar_url(wheel_url);
        assert!(sidecar.ends_with(".whl.metadata"));
    }

    #[test]
    fn realistic_pep503_html_attribute() {
        let attr = "sha256=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let hint = parse_hint_attr(attr).unwrap();
        assert!(hint.is_available());
    }

    #[test]
    fn realistic_no_hint_falls_back_to_zip_tail() {
        // No PEP 658 attribute / field — caller must use range.rs.
        let json_hint = parse_hint_json(None).unwrap();
        assert!(!json_hint.is_available());
    }
}
