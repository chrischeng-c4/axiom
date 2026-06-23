// HANDWRITE-BEGIN gap="missing-generator:hand-written:46d753da" tracker="standardize-gap-projects-mamba-src-pkgmgr-installer-record-rs" reason="Existing hand-written code in projects/mamba/src/pkgmgr/installer/record.rs requires tracked generator coverage."
// decodes base64url with the `base64` crate, recomputes sha256 with `sha2::Sha256`,
// compares to recorded value. Exempts the RECORD file itself from hashing
// (its own row carries blank fields).

/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Schema
/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Logic
use std::fs;
use std::path::Path;

use base64::Engine;
use sha2::{Digest, Sha256};

use super::InstallerError;

/// One row of `*.dist-info/RECORD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEntry {
    pub path: String,
    pub sha256_b64url: Option<String>,
    pub size: Option<u64>,
}

/// Parse a RECORD body. Blank lines are skipped. Each row is `path,hash,size`
/// where the `hash` field is `sha256=<b64url>` or empty (RECORD self-row).
pub fn parse(text: &str) -> Result<Vec<RecordEntry>, InstallerError> {
    let mut out = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.splitn(3, ',');
        let Some(path) = fields.next() else {
            return Err(InstallerError::MalformedWheel {
                path: None,
                detail: format!("RECORD line {} missing path field", lineno + 1),
            });
        };
        let hash_field = fields.next().unwrap_or("");
        let size_field = fields.next().unwrap_or("");

        let sha256_b64url = if hash_field.is_empty() {
            None
        } else if let Some(rest) = hash_field.strip_prefix("sha256=") {
            Some(rest.to_string())
        } else {
            return Err(InstallerError::MalformedWheel {
                path: None,
                detail: format!(
                    "RECORD line {}: unsupported hash algorithm in '{}', only sha256= accepted",
                    lineno + 1,
                    hash_field
                ),
            });
        };

        let size = if size_field.is_empty() {
            None
        } else {
            Some(
                size_field
                    .parse::<u64>()
                    .map_err(|_| InstallerError::MalformedWheel {
                        path: None,
                        detail: format!(
                            "RECORD line {}: invalid size '{}'",
                            lineno + 1,
                            size_field
                        ),
                    })?,
            )
        };

        out.push(RecordEntry {
            path: path.to_string(),
            sha256_b64url,
            size,
        });
    }
    Ok(out)
}

/// Verify every RECORD entry's hash + size against the staged file at
/// `staging.join(entry.path)`. Skips the entry whose path equals the RECORD
/// file itself (PEP 376 — its row is intentionally blank).
pub fn verify(staging: &Path, entries: &[RecordEntry]) -> Result<(), InstallerError> {
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    for entry in entries {
        if entry.sha256_b64url.is_none() {
            continue;
        }
        let abs = staging.join(&entry.path);
        let bytes = match fs::read(&abs) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(InstallerError::RecordMissingFile {
                    path: abs,
                    detail: format!("listed in RECORD but absent on disk: {}", entry.path),
                });
            }
            Err(e) => {
                return Err(InstallerError::Io {
                    path: Some(abs),
                    detail: e.to_string(),
                });
            }
        };

        if let Some(expected_size) = entry.size {
            if bytes.len() as u64 != expected_size {
                return Err(InstallerError::RecordHashMismatch {
                    path: abs,
                    detail: format!(
                        "size mismatch: RECORD declares {}, on-disk is {}",
                        expected_size,
                        bytes.len()
                    ),
                });
            }
        }

        let expected = entry.sha256_b64url.as_deref().unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual = engine.encode(hasher.finalize());
        if actual != expected {
            return Err(InstallerError::RecordHashMismatch {
                path: abs,
                detail: format!(
                    "sha256 mismatch: RECORD declares '{}', computed '{}'",
                    expected, actual
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handles_self_row_and_size_field() {
        let text = "\
pkg/__init__.py,sha256=abc,12
pkg-1.0.dist-info/RECORD,,
";
        let entries = parse(text).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "pkg/__init__.py");
        assert_eq!(entries[0].sha256_b64url.as_deref(), Some("abc"));
        assert_eq!(entries[0].size, Some(12));
        assert_eq!(entries[1].sha256_b64url, None);
        assert_eq!(entries[1].size, None);
    }

    #[test]
    fn parse_rejects_unsupported_hash_algorithm() {
        let text = "f.py,md5=deadbeef,3\n";
        let err = parse(text).unwrap_err();
        assert!(matches!(err, InstallerError::MalformedWheel { .. }));
    }

    #[test]
    fn parse_rejects_non_numeric_size() {
        let text = "f.py,sha256=abc,not-a-number\n";
        let err = parse(text).unwrap_err();
        assert!(matches!(err, InstallerError::MalformedWheel { .. }));
    }
}
// HANDWRITE-END
