// pep700.rs — PEP 700 file-level extension fields on the JSON
// Simple Repository API.
//
// PEP 691 ("JSON Simple Index") defined the file record shape:
//   { "filename": "...", "url": "...", "hashes": {...}, ... }
//
// PEP 700 ("Additional fields for the Simple Repository API")
// extends each file record with three optional fields the resolver
// can use to short-circuit work:
//
//   * `requires-python` — a PEP 440 specifier string. Lets the
//     resolver drop wheels for incompatible Python versions before
//     touching METADATA (often the slowest step).
//
//   * `size` — file size in bytes. Lets the downloader pre-allocate
//     a buffer + skip an HTTP HEAD; also feeds the freshness/upgrade
//     module's "x MiB of downloads pending" UI.
//
//   * `upload-time` — RFC 3339 timestamp. Used by the freshness
//     module (Tick ~25) to surface stale wheels, and by
//     pip/uv-style "--exclude-newer DATE" filtering.
//
// This module is the read-side: parse the three fields off a JSON
// `serde_json::Value`. Write-side (rendering for a generated index)
// is out of scope — mamba is a client, not a server.

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// PEP 700 extension fields on a single Simple-JSON file record.
/// All three fields are optional per the spec.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Raw PEP 440 specifier string (e.g. `">=3.8,<3.13"`). Left
    /// unparsed here because the pep440 module owns specifier
    /// parsing; callers run `pep440::parse_specifier(s)` when they
    /// need the typed form.
    #[serde(rename = "requires-python")]
    pub requires_python: Option<String>,

    /// File size in bytes. PEP 700 specifies the field as a JSON
    /// integer; we store as `u64` since wheels and sdists can
    /// exceed 4 GiB (numpy + manylinux + CUDA libs).
    pub size: Option<u64>,

    /// Upload timestamp as an RFC 3339 / ISO 8601 string verbatim.
    /// Parsing into a typed `chrono::DateTime` is left to callers
    /// — the freshness module already has a typed helper, and we'd
    /// rather not pull `chrono` into this module solely for round-
    /// tripping a string we never modify.
    #[serde(rename = "upload-time")]
    pub upload_time: Option<String>,
}

impl FileMetadata {
    /// True iff at least one PEP 700 field is present. Lets callers
    /// skip the merge work entirely when the index hasn't been
    /// upgraded yet.
    pub fn is_empty(&self) -> bool {
        self.requires_python.is_none() && self.size.is_none() && self.upload_time.is_none()
    }

    /// True iff `upload_time` looks like a syntactically valid RFC
    /// 3339 timestamp (minimal smoke check — full parsing is the
    /// freshness module's job). Returns `false` when the field is
    /// absent.
    pub fn upload_time_looks_valid(&self) -> bool {
        let Some(s) = self.upload_time.as_deref() else {
            return false;
        };
        // Heuristic shape: `YYYY-MM-DDTHH:MM:SS[.fff][Z|±HH:MM]`.
        // The first 19 ASCII chars must match `DDDD-DD-DDTDD:DD:DD`.
        let bytes = s.as_bytes();
        if bytes.len() < 19 {
            return false;
        }
        let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
        for (i, sep) in separators {
            if bytes[i] != sep {
                return false;
            }
        }
        for (i, b) in bytes[..19].iter().enumerate() {
            let is_sep = matches!(i, 4 | 7 | 10 | 13 | 16);
            if !is_sep && !b.is_ascii_digit() {
                return false;
            }
        }
        true
    }
}

/// Parse PEP 700 fields off a `serde_json::Value` representing one
/// file record. Unknown JSON shapes return `Ok(FileMetadata::default())`
/// — callers should treat the absence of any field as "index hasn't
/// adopted PEP 700 yet".
///
/// Type mismatches (e.g. `"size": "abc"`) surface as a typed
/// `ParseError` so the resolver can refuse to trust the index, but
/// missing fields are not an error.
pub fn parse_file_metadata(value: &serde_json::Value) -> Result<FileMetadata, IndexError> {
    let Some(obj) = value.as_object() else {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "PEP 700 file record must be a JSON object, got {}",
                json_kind(value)
            ),
        });
    };

    let requires_python = match obj.get("requires-python") {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(other) => {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "PEP 700 `requires-python` must be a string, got {}",
                    json_kind(other)
                ),
            });
        }
    };

    let size = match obj.get("size") {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::Number(n)) => match n.as_u64() {
            Some(v) => Some(v),
            None => {
                return Err(IndexError::ParseError {
                    url: String::new(),
                    detail: format!("PEP 700 `size` must be a non-negative integer, got {n}"),
                });
            }
        },
        Some(other) => {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!("PEP 700 `size` must be a number, got {}", json_kind(other)),
            });
        }
    };

    let upload_time = match obj.get("upload-time") {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(other) => {
            return Err(IndexError::ParseError {
                url: String::new(),
                detail: format!(
                    "PEP 700 `upload-time` must be a string, got {}",
                    json_kind(other)
                ),
            });
        }
    };

    Ok(FileMetadata {
        requires_python,
        size,
        upload_time,
    })
}

fn json_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- happy paths --------------------------------------------------

    #[test]
    fn parse_all_three_fields() {
        let v = json!({
            "filename": "pkg-1.0-py3-none-any.whl",
            "url": "https://x.example/pkg-1.0-py3-none-any.whl",
            "requires-python": ">=3.8,<3.13",
            "size": 1234567,
            "upload-time": "2025-01-15T10:30:00.000000Z"
        });
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.requires_python.as_deref(), Some(">=3.8,<3.13"));
        assert_eq!(meta.size, Some(1234567));
        assert_eq!(
            meta.upload_time.as_deref(),
            Some("2025-01-15T10:30:00.000000Z")
        );
        assert!(!meta.is_empty());
    }

    #[test]
    fn parse_only_requires_python() {
        let v = json!({"requires-python": ">=3.10"});
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.requires_python.as_deref(), Some(">=3.10"));
        assert_eq!(meta.size, None);
        assert_eq!(meta.upload_time, None);
    }

    #[test]
    fn parse_only_size() {
        let v = json!({"size": 9001});
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.size, Some(9001));
    }

    #[test]
    fn parse_only_upload_time() {
        let v = json!({"upload-time": "2024-06-30T12:00:00Z"});
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.upload_time.as_deref(), Some("2024-06-30T12:00:00Z"));
    }

    #[test]
    fn empty_object_parses_as_default() {
        let meta = parse_file_metadata(&json!({})).unwrap();
        assert!(meta.is_empty());
        assert_eq!(meta, FileMetadata::default());
    }

    #[test]
    fn extra_fields_are_ignored() {
        // PEP 700 doesn't say "reject unknown fields" — it adds
        // optional ones. Other ecosystem extensions (PEP 691 hashes,
        // yanked, dist-info-metadata) must round-trip through us
        // untouched.
        let v = json!({
            "filename": "x.whl",
            "url": "https://x.example/x.whl",
            "hashes": {"sha256": "abc"},
            "yanked": false,
            "requires-python": ">=3.7"
        });
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.requires_python.as_deref(), Some(">=3.7"));
    }

    #[test]
    fn explicit_null_fields_are_treated_as_absent() {
        // PEP 700 says "if not provided, missing or null". Both
        // shapes map to None.
        let v = json!({
            "requires-python": null,
            "size": null,
            "upload-time": null
        });
        let meta = parse_file_metadata(&v).unwrap();
        assert!(meta.is_empty());
    }

    #[test]
    fn size_zero_is_legal() {
        // Empty file is technically valid per the spec — don't
        // confuse zero with "absent".
        let v = json!({"size": 0});
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.size, Some(0));
        assert!(!meta.is_empty());
    }

    #[test]
    fn size_very_large_fits_u64() {
        // ~4 GiB + a bit, exercising the > u32 path.
        let v = json!({"size": 5_000_000_000u64});
        let meta = parse_file_metadata(&v).unwrap();
        assert_eq!(meta.size, Some(5_000_000_000));
    }

    // ---- rejection paths ----------------------------------------------

    #[test]
    fn non_object_value_errors() {
        let err = parse_file_metadata(&json!([1, 2, 3])).unwrap_err();
        assert!(err_detail(err).contains("must be a JSON object"));
    }

    #[test]
    fn requires_python_wrong_type_errors() {
        let v = json!({"requires-python": 3.10});
        let err = parse_file_metadata(&v).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("`requires-python`"));
        assert!(detail.contains("must be a string"));
    }

    #[test]
    fn size_negative_errors() {
        let v = json!({"size": -1});
        let err = parse_file_metadata(&v).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("`size`"));
        assert!(detail.contains("non-negative integer"));
    }

    #[test]
    fn size_float_errors() {
        let v = json!({"size": 3.5});
        let err = parse_file_metadata(&v).unwrap_err();
        let detail = err_detail(err);
        // serde_json::Value::Number with a float doesn't have an
        // `as_u64`, so it surfaces as the integer rejection.
        assert!(detail.contains("`size`"));
    }

    #[test]
    fn size_string_errors() {
        let v = json!({"size": "1234"});
        let err = parse_file_metadata(&v).unwrap_err();
        assert!(err_detail(err).contains("must be a number"));
    }

    #[test]
    fn upload_time_wrong_type_errors() {
        let v = json!({"upload-time": 1234567890});
        let err = parse_file_metadata(&v).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("`upload-time`"));
        assert!(detail.contains("must be a string"));
    }

    // ---- upload_time_looks_valid --------------------------------------

    #[test]
    fn upload_time_looks_valid_for_canonical_form() {
        let m = FileMetadata {
            upload_time: Some("2025-01-15T10:30:00".into()),
            ..Default::default()
        };
        assert!(m.upload_time_looks_valid());
    }

    #[test]
    fn upload_time_looks_valid_with_micros_and_z() {
        let m = FileMetadata {
            upload_time: Some("2025-01-15T10:30:00.123456Z".into()),
            ..Default::default()
        };
        assert!(m.upload_time_looks_valid());
    }

    #[test]
    fn upload_time_looks_valid_with_offset() {
        let m = FileMetadata {
            upload_time: Some("2025-01-15T10:30:00+09:00".into()),
            ..Default::default()
        };
        assert!(m.upload_time_looks_valid());
    }

    #[test]
    fn upload_time_invalid_too_short() {
        let m = FileMetadata {
            upload_time: Some("2025-01-15".into()),
            ..Default::default()
        };
        assert!(!m.upload_time_looks_valid());
    }

    #[test]
    fn upload_time_invalid_separator() {
        let m = FileMetadata {
            upload_time: Some("2025/01/15T10:30:00Z".into()),
            ..Default::default()
        };
        assert!(!m.upload_time_looks_valid());
    }

    #[test]
    fn upload_time_invalid_non_digit() {
        let m = FileMetadata {
            upload_time: Some("YYYY-01-15T10:30:00Z".into()),
            ..Default::default()
        };
        assert!(!m.upload_time_looks_valid());
    }

    #[test]
    fn upload_time_absent_means_not_valid() {
        let m = FileMetadata::default();
        assert!(!m.upload_time_looks_valid());
    }

    // ---- round-trip serde ---------------------------------------------

    #[test]
    fn serde_roundtrip_preserves_field_names() {
        let m = FileMetadata {
            requires_python: Some(">=3.10".into()),
            size: Some(42),
            upload_time: Some("2025-01-15T10:30:00Z".into()),
        };
        let s = serde_json::to_string(&m).unwrap();
        // Verify kebab-case wire form via serde rename.
        assert!(s.contains("\"requires-python\""));
        assert!(s.contains("\"upload-time\""));
        let back: FileMetadata = serde_json::from_str(&s).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn is_empty_round_trips_default() {
        let m = FileMetadata::default();
        assert!(m.is_empty());
        let s = serde_json::to_string(&m).unwrap();
        let back: FileMetadata = serde_json::from_str(&s).unwrap();
        assert!(back.is_empty());
    }
}
