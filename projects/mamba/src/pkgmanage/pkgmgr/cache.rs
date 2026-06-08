// REQ: on-disk cache layer for metadata and artifacts
// Spec §Cache Layout: disk layout, TTL, name normalization, atomic writes, sha256 sidecar.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

use crate::pkgmanage::pkgmgr::types::{FileHash, IndexError, PackageMetadata};

/// Metadata TTL in seconds (5 minutes per spec §Cache Layout).
pub const METADATA_TTL_SECS: u64 = 300;

/// Return the default cache directory for mamba artifacts.
///
/// Resolution order (per spec §Cache Layout):
/// 1. `$XDG_CACHE_HOME/mamba` if set and non-empty.
/// 2. `$HOME/.cache/mamba` if set and non-empty.
/// 3. `std::env::temp_dir()/mamba-cache` as last resort.
///
/// No filesystem side-effects — purely path computation.
pub fn default_cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("mamba");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".cache").join("mamba");
        }
    }
    std::env::temp_dir().join("mamba-cache")
}

/// Normalize a package name per PEP 503: lowercase and replace runs of
/// `[-_.]` with a single `-`.
pub fn normalize_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let re = regex::Regex::new(r"[-_.]+").expect("static regex");
    re.replace_all(&lower, "-").into_owned()
}

/// Compute the path for a cached metadata file.
///
/// Layout: `{cache_dir}/metadata/{normalized_name}/{source}-api.json`
/// where `source` ∈ `{"json", "simple"}`.
pub fn metadata_path(cache_dir: &Path, name: &str, source: &str) -> PathBuf {
    let normalized = normalize_name(name);
    cache_dir
        .join("metadata")
        .join(normalized)
        .join(format!("{source}-api.json"))
}

/// Compute the path for a cached artifact file.
///
/// Layout: `{cache_dir}/artifacts/{normalized_name}/{filename}`.
/// The sha256 sidecar lives at `{path}.sha256`.
pub fn artifact_path(cache_dir: &Path, name: &str, filename: &str) -> PathBuf {
    let normalized = normalize_name(name);
    cache_dir.join("artifacts").join(normalized).join(filename)
}

/// Compute the content-addressed cache path for an artifact identified by
/// its sha256 digest.
///
/// Layout: `{cache_dir}/content/{first_2}/{full_sha}` — the two-char prefix
/// keeps any single directory bounded as the cache grows into thousands of
/// wheels. uv uses the same shard-by-prefix pattern; the on-disk layout is
/// compatible across projects pointing at the same `MAMBA_CACHE_DIR`.
///
/// `expected_sha` must be a 64-char lowercase hex sha256. Inputs that are
/// shorter, malformed, or non-hex collapse to an unshared `_invalid/{raw}`
/// bucket so callers never panic on missing-sha inputs.
pub fn content_addressed_path(cache_dir: &Path, expected_sha: &str) -> PathBuf {
    let sha = expected_sha.trim().to_lowercase();
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return cache_dir.join("content").join("_invalid").join(sha);
    }
    let (prefix, _) = sha.split_at(2);
    cache_dir.join("content").join(prefix).join(&sha)
}

/// Verified content-addressed cache hit. Returns the artifact path when:
/// - `{cache_dir}/content/{prefix}/{sha}` exists,
/// - its bytes hash to `expected_sha`.
///
/// Returns `None` on any miss; deletes a corrupt entry before returning so
/// the next download path can re-stage a clean copy.
pub async fn read_content_addressed_artifact(
    cache_dir: &Path,
    expected_sha: &str,
) -> Option<PathBuf> {
    let path = content_addressed_path(cache_dir, expected_sha);
    if !path.exists() {
        return None;
    }
    let mut file = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(_) => {
            let _ = tokio::fs::remove_file(&path).await;
            return None;
        }
    };
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 65536];
    loop {
        match file.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => {
                drop(file);
                let _ = tokio::fs::remove_file(&path).await;
                return None;
            }
        }
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_sha.to_lowercase() {
        drop(file);
        let _ = tokio::fs::remove_file(&path).await;
        return None;
    }
    Some(path)
}

/// Promote a verified artifact at `verified_src` (e.g. the name-addressed
/// cache path just written by `download_artifact`) into the content-
/// addressed store at `{cache_dir}/content/{prefix}/{sha}`.
///
/// Uses hard-link first (zero copy, shared inode) and falls back to a byte
/// copy on cross-device or filesystem-rejection errors. Existing CAS entries
/// are left untouched — the digest already pins immutability. Errors during
/// the promotion step are non-fatal: callers see a best-effort warning via
/// IndexError::CacheIo, but the verified name-addressed copy is still good.
pub async fn promote_to_content_addressed(
    cache_dir: &Path,
    verified_src: &Path,
    expected_sha: &str,
) -> Result<PathBuf, IndexError> {
    let dest = content_addressed_path(cache_dir, expected_sha);
    if dest.exists() {
        return Ok(dest);
    }
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| IndexError::CacheIo {
            path: parent.display().to_string(),
            detail: format!("create CAS dir: {e}"),
        })?;
    }
    // Hard link first; fall back to a byte copy on EXDEV / unsupported FS.
    let hardlink_result = {
        let src = verified_src.to_path_buf();
        let dst = dest.clone();
        tokio::task::spawn_blocking(move || std::fs::hard_link(&src, &dst))
            .await
            .map_err(|e| IndexError::CacheIo {
                path: dest.display().to_string(),
                detail: format!("hard_link join error: {e}"),
            })?
    };
    if hardlink_result.is_err() {
        tokio::fs::copy(verified_src, &dest).await.map_err(|e| IndexError::CacheIo {
            path: dest.display().to_string(),
            detail: format!("CAS fallback copy: {e}"),
        })?;
    }
    Ok(dest)
}

/// Try to read cached metadata if it exists and is within TTL.
///
/// Returns `Some(meta)` when:
/// - the file exists,
/// - its mtime is within `ttl_secs` of now,
/// - and the JSON deserializes without error.
///
/// Never returns an error — stale / missing / corrupt cache → `None`.
pub async fn read_cached_metadata(
    cache_dir: &Path,
    name: &str,
    source: &str,
    ttl_secs: u64,
) -> Option<PackageMetadata> {
    let path = metadata_path(cache_dir, name, source);

    // Stat the file to check mtime.
    let metadata = tokio::fs::metadata(&path).await.ok()?;
    let mtime = metadata.modified().ok()?;
    let age = SystemTime::now().duration_since(mtime).unwrap_or(Duration::MAX);
    if age > Duration::from_secs(ttl_secs) {
        return None;
    }

    // Read and deserialize.
    let bytes = tokio::fs::read(&path).await.ok()?;
    serde_json::from_slice::<PackageMetadata>(&bytes).ok()
}

/// Write metadata to the cache using an atomic write (write .tmp then rename).
///
/// Creates parent directories as needed.
///
/// # Errors
///
/// Returns `IndexError::CacheIo` on any I/O failure.
pub async fn write_cached_metadata(
    cache_dir: &Path,
    name: &str,
    source: &str,
    meta: &PackageMetadata,
) -> Result<(), IndexError> {
    let path = metadata_path(cache_dir, name, source);
    let tmp_path = path.with_extension("json.tmp");

    // Create parent directories.
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| IndexError::CacheIo {
            path: parent.display().to_string(),
            detail: format!("create_dir_all failed: {e}"),
        })?;
    }

    // Serialize.
    let bytes = serde_json::to_vec(meta).map_err(|e| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: format!("JSON serialization failed: {e}"),
    })?;

    // Write to .tmp file.
    tokio::fs::write(&tmp_path, &bytes).await.map_err(|e| IndexError::CacheIo {
        path: tmp_path.display().to_string(),
        detail: format!("write .tmp failed: {e}"),
    })?;

    // Atomic rename.
    tokio::fs::rename(&tmp_path, &path).await.map_err(|e| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: format!("rename .tmp → final failed: {e}"),
    })?;

    Ok(())
}

/// Compute the path for a cached ETag sidecar file.
///
/// Returns `{metadata_path}.etag` — pure path computation, no IO.
pub fn etag_path(cache_dir: &Path, name: &str, source: &str) -> PathBuf {
    let base = metadata_path(cache_dir, name, source);
    PathBuf::from(format!("{}.etag", base.display()))
}

/// Read the cached ETag for a package metadata entry.
///
/// Returns `Some(etag)` when the `.etag` sidecar exists and contains valid UTF-8.
/// Returns `None` on any miss or I/O / encoding error — never errors.
/// Trailing whitespace/newlines are trimmed before returning.
pub async fn read_cached_etag(cache_dir: &Path, name: &str, source: &str) -> Option<String> {
    let path = etag_path(cache_dir, name, source);
    let bytes = tokio::fs::read(&path).await.ok()?;
    let s = String::from_utf8(bytes).ok()?;
    let trimmed = s.trim_end().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Write an ETag to the cache sidecar using an atomic write (`.etag.tmp` → rename).
///
/// Creates parent directories as needed.
///
/// # Errors
///
/// Returns `IndexError::CacheIo` on any I/O failure.
pub async fn write_cached_etag(
    cache_dir: &Path,
    name: &str,
    source: &str,
    etag: &str,
) -> Result<(), IndexError> {
    let path = etag_path(cache_dir, name, source);
    let tmp_path = PathBuf::from(format!("{}.tmp", path.display()));

    // Create parent directories.
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| IndexError::CacheIo {
            path: parent.display().to_string(),
            detail: format!("create_dir_all failed: {e}"),
        })?;
    }

    // Write to .tmp file.
    tokio::fs::write(&tmp_path, etag.as_bytes()).await.map_err(|e| IndexError::CacheIo {
        path: tmp_path.display().to_string(),
        detail: format!("write .tmp failed: {e}"),
    })?;

    // Atomic rename.
    tokio::fs::rename(&tmp_path, &path).await.map_err(|e| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: format!("rename .tmp → final failed: {e}"),
    })?;

    Ok(())
}

/// Try to return the cached artifact path if cache is valid (file + sidecar present,
/// sidecar hash matches `expected_hash.digest`, and actual bytes match sidecar).
///
/// Returns `Some(path)` on verified hit, `None` on any miss or corruption.
/// Deletes a corrupt cached file before returning `None`.
pub async fn read_cached_artifact(
    cache_dir: &Path,
    name: &str,
    filename: &str,
    expected_hash: &FileHash,
) -> Option<PathBuf> {
    let path = artifact_path(cache_dir, name, filename);
    let sidecar = PathBuf::from(format!("{}.sha256", path.display()));

    // Check both file and sidecar exist.
    if !path.exists() || !sidecar.exists() {
        return None;
    }

    // Read sidecar digest.
    let sidecar_digest = match tokio::fs::read_to_string(&sidecar).await {
        Ok(s) => s.trim().to_lowercase(),
        Err(_) => {
            let _ = tokio::fs::remove_file(&path).await;
            let _ = tokio::fs::remove_file(&sidecar).await;
            return None;
        }
    };

    // Compare sidecar to expected.
    if sidecar_digest != expected_hash.digest.to_lowercase() {
        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_file(&sidecar).await;
        return None;
    }

    // Verify actual file bytes against the sidecar digest (defense against corruption).
    let mut file = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(_) => {
            let _ = tokio::fs::remove_file(&path).await;
            let _ = tokio::fs::remove_file(&sidecar).await;
            return None;
        }
    };

    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 65536];
    loop {
        match file.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => {
                drop(file);
                let _ = tokio::fs::remove_file(&path).await;
                let _ = tokio::fs::remove_file(&sidecar).await;
                return None;
            }
        }
    }
    let actual_digest = format!("{:x}", hasher.finalize());

    if actual_digest != sidecar_digest {
        drop(file);
        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_file(&sidecar).await;
        return None;
    }

    Some(path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Write;

    use super::*;
    use crate::pkgmanage::pkgmgr::types::{FileHash, PackageMetadata, ReleaseFile};

    fn make_meta(name: &str) -> PackageMetadata {
        PackageMetadata {
            name: name.to_string(),
            versions: vec!["1.0.0".to_string()],
            releases: {
                let mut m = BTreeMap::new();
                m.insert(
                    "1.0.0".to_string(),
                    vec![ReleaseFile {
                        filename: format!("{name}-1.0.0-py3-none-any.whl"),
                        url: format!("https://example.com/{name}-1.0.0.whl"),
                        hash: FileHash {
                            algorithm: "sha256".to_string(),
                            digest: "a".repeat(64),
                        },
                        requires_python: Some(">=3.8".to_string()),
                        size: None,
                        upload_time: None,
                        yanked: false,
                        yanked_reason: None,
                        dist_info_metadata: serde_json::Value::Null,
                        source: Some("json-api".to_string()),
                    }],
                );
                m
            },
            requires_python: Some(">=3.8".to_string()),
            source: "json-api".to_string(),
        }
    }

    // REQ: PEP 503 name normalization — lowercase + collapse [-_.] runs to single `-`.
    #[test]
    fn test_normalize_name_pep503_cases() {
        assert_eq!(normalize_name("Foo-Bar"), "foo-bar");
        assert_eq!(normalize_name("foo_bar.baz"), "foo-bar-baz");
        assert_eq!(normalize_name("A.B_C--D"), "a-b-c-d");
        assert_eq!(normalize_name("Already-Lower"), "already-lower");
        assert_eq!(normalize_name("___..."), "-");
    }

    // REQ: test_metadata_roundtrip_under_ttl — write then read returns Some with equal data
    #[tokio::test]
    async fn test_metadata_roundtrip_under_ttl() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();
        let meta = make_meta("requests");

        write_cached_metadata(cache_dir, "requests", "json", &meta)
            .await
            .expect("write should succeed");

        // TTL = 300 s — file was just written, age ≈ 0
        let result = read_cached_metadata(cache_dir, "requests", "json", 300).await;
        assert!(result.is_some(), "should return Some under TTL");
        let loaded = result.unwrap();
        assert_eq!(loaded.name, meta.name);
        assert_eq!(loaded.versions, meta.versions);
        assert_eq!(loaded.source, meta.source);
    }

    // REQ: test_metadata_stale_ttl_returns_none — TTL=0 means any file is stale
    #[tokio::test]
    async fn test_metadata_stale_ttl_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();
        let meta = make_meta("requests");

        write_cached_metadata(cache_dir, "requests", "json", &meta)
            .await
            .expect("write should succeed");

        // TTL = 0 → any age causes miss.
        let result = read_cached_metadata(cache_dir, "requests", "json", 0).await;
        assert!(result.is_none(), "should return None when TTL=0 (always stale)");
    }

    // REQ: test_metadata_corrupt_file_returns_none — garbage JSON → None, no panic
    #[tokio::test]
    async fn test_metadata_corrupt_file_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();

        // Create the parent dir and write raw garbage to the cache path.
        let path = metadata_path(cache_dir, "mypkg", "json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"this is not valid JSON {{{}}}").unwrap();

        let result = read_cached_metadata(cache_dir, "mypkg", "json", 9999).await;
        assert!(result.is_none(), "corrupt JSON should return None, not panic");
    }

    // REQ: test_artifact_sidecar_verify_hit — correct sidecar → Some(path)
    #[tokio::test]
    async fn test_artifact_sidecar_verify_hit() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();

        let file_contents = b"fake wheel bytes for testing";
        let filename = "mypkg-1.0.0-py3-none-any.whl";

        // Compute the real sha256.
        let mut hasher = Sha256::new();
        hasher.update(file_contents);
        let digest = format!("{:x}", hasher.finalize());

        // Write the artifact file and sidecar.
        let path = artifact_path(cache_dir, "mypkg", filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, file_contents).unwrap();
        let sidecar = PathBuf::from(format!("{}.sha256", path.display()));
        std::fs::write(&sidecar, &digest).unwrap();

        let expected_hash = FileHash {
            algorithm: "sha256".to_string(),
            digest: digest.clone(),
        };

        let result = read_cached_artifact(cache_dir, "mypkg", filename, &expected_hash).await;
        assert!(result.is_some(), "correct sidecar + bytes should return Some");
        assert_eq!(result.unwrap(), path);
    }

    // REQ: AC4 — ETag sidecar round-trip: write then read returns same value; miss returns None.
    #[tokio::test]
    async fn test_etag_cache_round_trip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();

        // Hit case: write then read back.
        write_cached_etag(cache_dir, "requests", "json", "W/\"abc123\"")
            .await
            .expect("write should succeed");
        let result = read_cached_etag(cache_dir, "requests", "json").await;
        assert_eq!(result, Some("W/\"abc123\"".to_string()));

        // Miss case: package not written yet.
        let miss = read_cached_etag(cache_dir, "nonexistent", "json").await;
        assert_eq!(miss, None);
    }

    // REQ: Tick 17 — content_addressed_path follows the {prefix}/{full_sha} layout
    #[test]
    fn test_content_addressed_path_layout() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = tmp.path();
        let sha = "abcdef0123456789".repeat(4); // 64 hex chars
        let path = content_addressed_path(cache, &sha);
        let expected = cache.join("content").join("ab").join(&sha);
        assert_eq!(path, expected);
    }

    // REQ: Tick 17 — malformed shas collapse into the _invalid bucket, not panic
    #[test]
    fn test_content_addressed_path_invalid_inputs() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = tmp.path();

        let short = content_addressed_path(cache, "deadbeef");
        assert_eq!(short, cache.join("content").join("_invalid").join("deadbeef"));

        let non_hex = "z".repeat(64);
        let nh = content_addressed_path(cache, &non_hex);
        assert_eq!(nh, cache.join("content").join("_invalid").join(&non_hex));

        let empty = content_addressed_path(cache, "");
        assert_eq!(empty, cache.join("content").join("_invalid").join(""));
    }

    // REQ: Tick 17 — content_addressed_path is case-insensitive (canonicalizes to lowercase)
    #[test]
    fn test_content_addressed_path_case_insensitive() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache = tmp.path();
        let upper = "ABCDEF0123456789".repeat(4); // 64 hex, uppercase
        let lower = upper.to_lowercase();
        let p_upper = content_addressed_path(cache, &upper);
        let p_lower = content_addressed_path(cache, &lower);
        assert_eq!(p_upper, p_lower);
    }

    // REQ: Tick 17 — promote_to_content_addressed round-trips and is reusable
    #[tokio::test]
    async fn test_promote_to_content_addressed_round_trip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();

        // Stage a verified source file.
        let contents = b"verified wheel bytes for CAS promote test";
        let mut hasher = Sha256::new();
        hasher.update(contents);
        let digest = format!("{:x}", hasher.finalize());

        let src = cache_dir.join("staging").join("payload.whl");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, contents).unwrap();

        // First promote — should create CAS entry.
        let cas_path = promote_to_content_addressed(cache_dir, &src, &digest)
            .await
            .expect("first promote should succeed");
        assert!(cas_path.exists(), "CAS path must exist after promote");

        // Bytes at CAS path must match.
        let cas_bytes = tokio::fs::read(&cas_path).await.unwrap();
        assert_eq!(&cas_bytes[..], &contents[..]);

        // Layout sanity: {cache}/content/{prefix}/{sha}
        let expected = content_addressed_path(cache_dir, &digest);
        assert_eq!(cas_path, expected);

        // Second promote — must be a no-op (idempotent), still succeeds.
        let cas_path2 = promote_to_content_addressed(cache_dir, &src, &digest)
            .await
            .expect("second promote should succeed");
        assert_eq!(cas_path, cas_path2);

        // Read-back through the verified accessor must return Some(path).
        let hit = read_content_addressed_artifact(cache_dir, &digest).await;
        assert_eq!(hit, Some(cas_path));
    }

    // REQ: Tick 17 — corrupt CAS entry (bytes don't match digest) → None + deleted
    #[tokio::test]
    async fn test_read_content_addressed_artifact_deletes_corrupt_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();

        // Plant a corrupt entry: real digest of contents X, file body Y.
        let real_contents = b"expected payload";
        let mut hasher = Sha256::new();
        hasher.update(real_contents);
        let digest = format!("{:x}", hasher.finalize());

        let cas_path = content_addressed_path(cache_dir, &digest);
        std::fs::create_dir_all(cas_path.parent().unwrap()).unwrap();
        // Write GARBAGE bytes, not the real contents.
        std::fs::write(&cas_path, b"tampered bytes do not match digest").unwrap();

        let hit = read_content_addressed_artifact(cache_dir, &digest).await;
        assert!(hit.is_none(), "tampered CAS entry must miss");
        assert!(!cas_path.exists(), "corrupt entry must be deleted");
    }

    // REQ: Tick 17 — miss when CAS path doesn't exist at all
    #[tokio::test]
    async fn test_read_content_addressed_artifact_miss_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();
        let digest = "f".repeat(64);
        let hit = read_content_addressed_artifact(cache_dir, &digest).await;
        assert!(hit.is_none());
    }

    // REQ: test_artifact_sidecar_mismatch_deletes_and_returns_none
    #[tokio::test]
    async fn test_artifact_sidecar_mismatch_deletes_and_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cache_dir = tmp.path();

        let file_contents = b"real wheel bytes";
        let filename = "mypkg-2.0.0-py3-none-any.whl";

        let path = artifact_path(cache_dir, "mypkg", filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, file_contents).unwrap();

        // Sidecar contains WRONG digest.
        let wrong_digest = "b".repeat(64);
        let sidecar = PathBuf::from(format!("{}.sha256", path.display()));
        std::fs::write(&sidecar, &wrong_digest).unwrap();

        let expected_hash = FileHash {
            algorithm: "sha256".to_string(),
            digest: wrong_digest,
        };

        let result = read_cached_artifact(cache_dir, "mypkg", filename, &expected_hash).await;
        assert!(result.is_none(), "sidecar mismatch against expected should return None");

        // File should be deleted.
        assert!(!path.exists(), "corrupt artifact file should be deleted");
    }
}
