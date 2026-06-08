// PEP 376 RECORD generation (Tick 35).
//
// `installer::record` already parses + verifies RECORD; the *writer*
// half (hash a staged tree and emit the CSV body) lived only inside
// the wheel installer's monolith. This tick lifts it into a focused,
// reusable module so:
//   * `mamba build` can emit a RECORD when constructing wheels from
//     source,
//   * `mamba install --from-tree` can pre-stage a RECORD for an
//     unpacked sdist,
//   * the regenerable layer can re-emit a RECORD after fixups.
//
// What this module provides:
//   * `hash_path(file_path)`  — sha256+size for a single file,
//     returned in PEP 376's `sha256=BASE64URL_NOPAD` form.
//   * `walk_and_hash(root)`   — recursive walk producing one
//     RecordEntryDraft per regular file. Hidden directories are NOT
//     skipped (PEP 376 records *everything* installed).
//   * `render_record(...)`    — RFC 4180-quoted CSV emission, with the
//     RECORD file's own row inserted as a blank-hash/blank-size entry
//     (PEP 376 requirement).
//
// Pure data + filesystem read. No network, no subprocess.

use std::path::{Path, PathBuf};

use base64::Engine;
use sha2::{Digest, Sha256};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One row to be written to RECORD, before final string formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEntryDraft {
    /// Forward-slash relative path from the install root. PEP 376
    /// stipulates `/` separators on every platform.
    pub path: String,
    /// `sha256=BASE64URL_NOPAD` for the file content. `None` only for
    /// the RECORD file's self-row.
    pub sha256_b64url: Option<String>,
    /// File size in bytes. `None` only for the RECORD self-row.
    pub size: Option<u64>,
}

/// Hash + measure a single file. Returns `(sha256_b64url, size_bytes)`.
pub fn hash_path(file_path: &Path) -> Result<(String, u64), IndexError> {
    let bytes = std::fs::read(file_path).map_err(|err| IndexError::CacheIo {
        path: file_path.display().to_string(),
        detail: format!("reading file for RECORD hash: {err}"),
    })?;
    Ok(hash_bytes(&bytes))
}

/// Hash + measure an in-memory blob. Same encoding as `hash_path`:
/// base64 URL_SAFE_NO_PAD of the sha256 digest, plus the byte length.
/// Used by the wheel builder which composes RECORD over data it has
/// not yet written to disk.
pub fn hash_bytes(bytes: &[u8]) -> (String, u64) {
    let size = bytes.len() as u64;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let digest = engine.encode(hasher.finalize());
    (digest, size)
}

/// Walk `root` recursively and hash every regular file. The returned
/// list is sorted by path so RECORD diffs stay stable across runs.
///
/// Symlinks are followed *only* if they point to regular files
/// (`is_file()` returns true after metadata resolution). Broken
/// symlinks are recorded as errors, not silently skipped.
pub fn walk_and_hash(root: &Path) -> Result<Vec<RecordEntryDraft>, IndexError> {
    if !root.exists() {
        return Err(IndexError::CacheIo {
            path: root.display().to_string(),
            detail: "RECORD walk root does not exist".into(),
        });
    }
    let mut entries: Vec<RecordEntryDraft> = Vec::new();
    walk_dir(root, root, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn walk_dir(root: &Path, dir: &Path, out: &mut Vec<RecordEntryDraft>) -> Result<(), IndexError> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| IndexError::CacheIo {
        path: dir.display().to_string(),
        detail: format!("reading directory: {err}"),
    })?;
    for entry in read_dir {
        let entry = entry.map_err(|err| IndexError::CacheIo {
            path: dir.display().to_string(),
            detail: format!("reading directory entry: {err}"),
        })?;
        let path = entry.path();
        let meta = std::fs::metadata(&path).map_err(|err| IndexError::CacheIo {
            path: path.display().to_string(),
            detail: format!("stat: {err}"),
        })?;
        if meta.is_dir() {
            walk_dir(root, &path, out)?;
        } else if meta.is_file() {
            let rel = relative_forward_slash(root, &path)?;
            let (sha, size) = hash_path(&path)?;
            out.push(RecordEntryDraft {
                path: rel,
                sha256_b64url: Some(sha),
                size: Some(size),
            });
        }
        // Other types (sockets, fifos) are not legal in a wheel/install
        // tree; we silently skip them and let downstream callers
        // discover the inconsistency.
    }
    Ok(())
}

fn relative_forward_slash(root: &Path, path: &Path) -> Result<String, IndexError> {
    let rel = path.strip_prefix(root).map_err(|_| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: format!("path is not under root {}", root.display()),
    })?;
    // PEP 376: forward slashes on every platform.
    let s: String = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    Ok(s)
}

/// Insert the RECORD self-row and render the CSV body.
///
/// `record_path_in_install` is the forward-slash path of the RECORD
/// file itself, relative to the install root — typically
/// `<distname>-<version>.dist-info/RECORD`. PEP 376 mandates the row
/// be present with blank hash + size fields (it can't hash itself).
pub fn render_record(
    drafts: &[RecordEntryDraft],
    record_path_in_install: &str,
) -> Result<String, IndexError> {
    let mut combined: Vec<RecordEntryDraft> = drafts
        .iter()
        .filter(|d| d.path != record_path_in_install)
        .cloned()
        .collect();
    combined.push(RecordEntryDraft {
        path: record_path_in_install.to_string(),
        sha256_b64url: None,
        size: None,
    });
    combined.sort_by(|a, b| a.path.cmp(&b.path));

    let mut out = String::new();
    for entry in combined {
        out.push_str(&csv_quote(&entry.path));
        out.push(',');
        if let Some(hash) = entry.sha256_b64url {
            out.push_str("sha256=");
            out.push_str(&hash);
        }
        out.push(',');
        if let Some(size) = entry.size {
            out.push_str(&size.to_string());
        }
        out.push('\n');
    }
    Ok(out)
}

/// RFC 4180 quoting: if a field contains comma, double-quote, CR, or
/// LF, surround it with double quotes and double any internal quote.
/// pip + uv both round-trip through this; we match.
fn csv_quote(field: &str) -> String {
    let needs_quote = field
        .chars()
        .any(|c| c == ',' || c == '"' || c == '\r' || c == '\n');
    if !needs_quote {
        return field.to_string();
    }
    let mut out = String::with_capacity(field.len() + 2);
    out.push('"');
    for c in field.chars() {
        if c == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(c);
        }
    }
    out.push('"');
    out
}

/// Convenience: walk + render in one call.
///
/// `install_root` is the directory whose children become RECORD paths.
/// `record_path_in_install` is the forward-slash RECORD location.
pub fn build_record(
    install_root: &Path,
    record_path_in_install: &str,
) -> Result<String, IndexError> {
    let drafts = walk_and_hash(install_root)?;
    render_record(&drafts, record_path_in_install)
}

/// Locate the RECORD file under a freshly-unpacked install root.
/// Returns the absolute path. Errors if zero or more than one
/// `*.dist-info/RECORD` is found — the wheel/install tree is malformed
/// in either case.
pub fn find_record_path(install_root: &Path) -> Result<PathBuf, IndexError> {
    let entries = std::fs::read_dir(install_root).map_err(|err| IndexError::CacheIo {
        path: install_root.display().to_string(),
        detail: format!("reading install root: {err}"),
    })?;
    let mut candidates: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| IndexError::CacheIo {
            path: install_root.display().to_string(),
            detail: format!("reading install root entry: {err}"),
        })?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|n| n.ends_with(".dist-info"))
            == Some(true)
        {
            let record = path.join("RECORD");
            if record.is_file() {
                candidates.push(record);
            }
        }
    }
    match candidates.len() {
        0 => Err(IndexError::CacheIo {
            path: install_root.display().to_string(),
            detail: "no *.dist-info/RECORD found under install root".into(),
        }),
        1 => Ok(candidates.into_iter().next().unwrap()),
        _ => Err(IndexError::CacheIo {
            path: install_root.display().to_string(),
            detail: format!(
                "multiple *.dist-info/RECORD files found: {:?}",
                candidates
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, body: &[u8]) {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, body).unwrap();
    }

    #[test]
    fn hash_path_matches_known_sha256_b64url() {
        // Known vector: sha256("hello") base64url-no-pad.
        let dir = TempDir::new().unwrap();
        write(dir.path(), "hello.txt", b"hello");
        let (sha, size) = hash_path(&dir.path().join("hello.txt")).unwrap();
        assert_eq!(size, 5);
        // sha256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        // base64url-no-pad:
        assert_eq!(sha, "LPJNul-wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ");
    }

    #[test]
    fn walk_and_hash_recurses_and_sorts() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "z/last.py", b"last\n");
        write(dir.path(), "a/first.py", b"first\n");
        write(dir.path(), "middle.py", b"middle\n");
        let entries = walk_and_hash(dir.path()).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["a/first.py", "middle.py", "z/last.py"]);
    }

    #[test]
    fn walk_and_hash_uses_forward_slashes() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "pkg/sub/mod.py", b"x\n");
        let entries = walk_and_hash(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "pkg/sub/mod.py");
    }

    #[test]
    fn walk_and_hash_errors_on_missing_root() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("does-not-exist");
        let err = walk_and_hash(&missing).unwrap_err();
        assert!(format!("{err}").contains("does not exist"));
    }

    #[test]
    fn render_record_inserts_self_row_with_blank_hash_and_size() {
        let drafts = vec![RecordEntryDraft {
            path: "pkg/__init__.py".into(),
            sha256_b64url: Some("abc".into()),
            size: Some(12),
        }];
        let body = render_record(&drafts, "pkg-1.0.dist-info/RECORD").unwrap();
        assert!(body.contains("pkg/__init__.py,sha256=abc,12\n"));
        assert!(body.contains("pkg-1.0.dist-info/RECORD,,\n"));
    }

    #[test]
    fn render_record_overrides_user_supplied_self_row_with_blank() {
        // If a caller passes an entry whose path equals the RECORD
        // path with a hash already filled in, we strip it and replace
        // with the canonical blank-hash row.
        let drafts = vec![
            RecordEntryDraft {
                path: "a.py".into(),
                sha256_b64url: Some("h".into()),
                size: Some(1),
            },
            RecordEntryDraft {
                path: "pkg-1.0.dist-info/RECORD".into(),
                sha256_b64url: Some("WRONG".into()),
                size: Some(99),
            },
        ];
        let body = render_record(&drafts, "pkg-1.0.dist-info/RECORD").unwrap();
        assert!(body.contains("pkg-1.0.dist-info/RECORD,,\n"));
        assert!(!body.contains("WRONG"));
    }

    #[test]
    fn render_record_csv_quotes_paths_with_commas() {
        let drafts = vec![RecordEntryDraft {
            path: "weird,name.py".into(),
            sha256_b64url: Some("h".into()),
            size: Some(1),
        }];
        let body = render_record(&drafts, "x.dist-info/RECORD").unwrap();
        assert!(body.contains("\"weird,name.py\",sha256=h,1\n"));
    }

    #[test]
    fn render_record_csv_quotes_paths_with_quotes() {
        let drafts = vec![RecordEntryDraft {
            path: "say\"hi.py".into(),
            sha256_b64url: Some("h".into()),
            size: Some(1),
        }];
        let body = render_record(&drafts, "x.dist-info/RECORD").unwrap();
        assert!(body.contains("\"say\"\"hi.py\",sha256=h,1\n"));
    }

    #[test]
    fn render_record_sorts_output() {
        let drafts = vec![
            RecordEntryDraft {
                path: "z.py".into(),
                sha256_b64url: Some("a".into()),
                size: Some(1),
            },
            RecordEntryDraft {
                path: "a.py".into(),
                sha256_b64url: Some("a".into()),
                size: Some(1),
            },
        ];
        let body = render_record(&drafts, "x.dist-info/RECORD").unwrap();
        let pos_a = body.find("a.py").unwrap();
        let pos_z = body.find("z.py").unwrap();
        assert!(pos_a < pos_z);
    }

    #[test]
    fn build_record_end_to_end() {
        let dir = TempDir::new().unwrap();
        write(dir.path(), "pkg/__init__.py", b"x = 1\n");
        write(dir.path(), "pkg-1.0.dist-info/METADATA", b"Name: pkg\n");
        // Pretend RECORD does not yet exist.
        let body = build_record(dir.path(), "pkg-1.0.dist-info/RECORD").unwrap();
        // Contains the two real files plus the self-row.
        assert!(body.contains("pkg/__init__.py,sha256="));
        assert!(body.contains("pkg-1.0.dist-info/METADATA,sha256="));
        assert!(body.contains("pkg-1.0.dist-info/RECORD,,\n"));
    }

    #[test]
    fn build_record_roundtrips_through_installer_parser() {
        // The hash/size we emit must be exactly what
        // `installer::record::parse` + `verify` expect.
        use crate::pkgmanage::pkgmgr::installer::record as record_parser;
        let dir = TempDir::new().unwrap();
        write(dir.path(), "pkg/__init__.py", b"hello\n");
        let body = build_record(dir.path(), "pkg-1.0.dist-info/RECORD").unwrap();
        let parsed = record_parser::parse(&body).expect("parse should succeed");
        // Should have the file row + the self-row.
        assert_eq!(parsed.len(), 2);
        let pkg = parsed
            .iter()
            .find(|e| e.path == "pkg/__init__.py")
            .expect("pkg row");
        assert!(pkg.sha256_b64url.is_some());
        assert_eq!(pkg.size, Some(6)); // "hello\n"
        let self_row = parsed
            .iter()
            .find(|e| e.path == "pkg-1.0.dist-info/RECORD")
            .expect("self-row");
        assert!(self_row.sha256_b64url.is_none());
        assert!(self_row.size.is_none());
    }

    #[test]
    fn find_record_path_locates_single_record() {
        let dir = TempDir::new().unwrap();
        let dist_info = dir.path().join("pkg-1.0.dist-info");
        fs::create_dir_all(&dist_info).unwrap();
        fs::write(dist_info.join("RECORD"), "").unwrap();
        let found = find_record_path(dir.path()).unwrap();
        assert_eq!(found, dist_info.join("RECORD"));
    }

    #[test]
    fn find_record_path_errors_on_zero_or_multiple() {
        let dir = TempDir::new().unwrap();
        let err = find_record_path(dir.path()).unwrap_err();
        assert!(format!("{err}").contains("no *.dist-info/RECORD"));

        // Two dist-info dirs.
        fs::create_dir_all(dir.path().join("a-1.0.dist-info")).unwrap();
        fs::write(dir.path().join("a-1.0.dist-info/RECORD"), "").unwrap();
        fs::create_dir_all(dir.path().join("b-2.0.dist-info")).unwrap();
        fs::write(dir.path().join("b-2.0.dist-info/RECORD"), "").unwrap();
        let err = find_record_path(dir.path()).unwrap_err();
        assert!(format!("{err}").contains("multiple"));
    }
}
