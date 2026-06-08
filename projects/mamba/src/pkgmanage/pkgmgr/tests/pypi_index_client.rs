#![cfg(test)]

// Real-network integration tests for the PyPI index client.
//
// These tests hit https://pypi.org/ directly and require an internet connection.
// They are gated behind the `integration` feature flag so that normal
// `cargo test` runs skip them entirely.
//
// Run with:
//   cargo test -p mamba --features integration -- --test-threads=1 pypi_index_client_integration
//
// Tests:
//   IT-01  fetch_metadata("requests")                   → Ok, non-empty version, ≥1 file with sha256
//   IT-02  fetch_metadata("<nonexistent>")               → Err(NotFound)
//   IT-03  fetch_metadata_simple("flask")               → Ok, non-empty files vec
//   IT-04  fetch_metadata_batch(3 names, 2)             → 3 results in order; slots 0+1 Ok, slot 2 Err
//   IT-05  download_artifact(<smallest wheel>)          → Ok, sha256 matches (end-to-end streaming verify)
//   IT-06  fetch_metadata("requests")                   → latest version is a PEP 440 release (not pre/dev) [AC2]
//   IT-07  fetch_metadata_simple("requests")            → Ok; ≥1 file with non-empty yanked_reason [AC1/R1/R2]

#![cfg(feature = "integration")]

use std::sync::atomic::{AtomicU64, Ordering};

use crate::pkgmanage::pkgmgr::{IndexClient, IndexError};

/// Create a fresh IndexClient pointing at real PyPI with a unique temp cache dir.
///
/// Each test gets its own isolated cache so tests cannot interfere with each other.
fn make_real_client() -> IndexClient {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    IndexClient {
        index_url: "https://pypi.org".to_string(),
        cache_dir: format!("/tmp/mamba-integration-cache-{nanos}-{n}"),
        max_concurrent: 4,
        timeout_secs: 30,
        retry_max: 3,
    }
}

// IT-01: fetch_metadata("requests") → Ok with non-empty version and ≥1 sha256 file.
#[tokio::test]
async fn it_01_fetch_metadata_requests_ok() {
    let client = make_real_client();
    let meta = client
        .fetch_metadata("requests")
        .await
        .expect("IT-01: fetch_metadata(requests) must succeed");

    assert!(!meta.versions.is_empty(), "IT-01: requests must have at least one version");
    assert_eq!(meta.name.to_lowercase(), "requests", "IT-01: name must be 'requests'");

    // At least one release file must have a sha256 hash.
    let has_sha256 = meta
        .releases
        .values()
        .flat_map(|files| files.iter())
        .any(|f| f.hash.algorithm == "sha256" && !f.hash.digest.is_empty());
    assert!(has_sha256, "IT-01: at least one release file must have a sha256 hash");
}

// IT-02: fetch_metadata("<nonexistent>") → Err(IndexError::NotFound).
#[tokio::test]
async fn it_02_fetch_metadata_nonexistent_not_found() {
    let client = make_real_client();
    let err = client
        .fetch_metadata("this-package-will-not-exist-xyz-12345")
        .await
        .expect_err("IT-02: nonexistent package must return an error");

    assert!(
        matches!(err, IndexError::NotFound { .. }),
        "IT-02: expected IndexError::NotFound, got: {:?}",
        err
    );
}

// IT-03: fetch_metadata_simple("flask") → Ok with non-empty files vec.
#[tokio::test]
async fn it_03_fetch_metadata_simple_flask_ok() {
    let client = make_real_client();
    let meta = client
        .fetch_metadata_simple("flask")
        .await
        .expect("IT-03: fetch_metadata_simple(flask) must succeed");

    let total_files: usize = meta.releases.values().map(|v| v.len()).sum();
    assert!(total_files > 0, "IT-03: flask must have at least one release file");
    assert_eq!(meta.source, "simple-api", "IT-03: source must be 'simple-api'");
}

// IT-04: fetch_metadata_batch with mixed results — 3 results in input order.
//   slot 0: "requests" → Ok
//   slot 1: "flask"    → Ok
//   slot 2: "<nonexistent>" → Err(NotFound)
#[tokio::test]
async fn it_04_fetch_metadata_batch_mixed() {
    let client = make_real_client();
    let names = vec![
        "requests".to_string(),
        "flask".to_string(),
        "this-package-will-not-exist-xyz-12345".to_string(),
    ];

    let results = client.fetch_metadata_batch(names.clone(), 2).await;

    assert_eq!(results.len(), 3, "IT-04: must return exactly 3 results");

    // Slot 0: requests → Ok
    assert_eq!(results[0].0, "requests", "IT-04: slot 0 name must be 'requests'");
    assert!(results[0].1.is_ok(), "IT-04: slot 0 (requests) must be Ok, got: {:?}", results[0].1);

    // Slot 1: flask → Ok
    assert_eq!(results[1].0, "flask", "IT-04: slot 1 name must be 'flask'");
    assert!(results[1].1.is_ok(), "IT-04: slot 1 (flask) must be Ok, got: {:?}", results[1].1);

    // Slot 2: nonexistent → Err(NotFound)
    assert_eq!(results[2].0, "this-package-will-not-exist-xyz-12345", "IT-04: slot 2 name mismatch");
    match &results[2].1 {
        Err(IndexError::NotFound { .. }) => {}
        other => panic!("IT-04: slot 2 must be NotFound, got: {:?}", other),
    }
}

// IT-05: download smallest wheel from requests → Ok, sha256 verified end-to-end.
#[tokio::test]
async fn it_05_download_smallest_requests_wheel_sha256_verified() {
    let client = make_real_client();

    // Fetch metadata to find real release files.
    let meta = client
        .fetch_metadata("requests")
        .await
        .expect("IT-05: must be able to fetch requests metadata");

    // Find the smallest wheel file that has a sha256 hash across all versions.
    let smallest_wheel = meta
        .releases
        .values()
        .flat_map(|files| files.iter())
        .filter(|f| {
            f.filename.ends_with(".whl")
                && f.hash.algorithm == "sha256"
                && !f.hash.digest.is_empty()
                && !f.yanked
        })
        .min_by_key(|f| f.size.unwrap_or(u64::MAX));

    let file = smallest_wheel.expect("IT-05: at least one non-yanked whl with sha256 must exist");

    // Download and verify.
    let path = client
        .download_artifact("requests", file)
        .await
        .expect("IT-05: download_artifact must succeed");

    assert!(path.exists(), "IT-05: downloaded file must exist on disk");

    // Verify the sha256 by re-hashing the file on disk.
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(&path).expect("IT-05: must be able to read downloaded file");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual = format!("{:x}", hasher.finalize());
    assert_eq!(
        actual,
        file.hash.digest.to_lowercase(),
        "IT-05: sha256 of downloaded file must match index-declared digest"
    );
}

// IT-06 [AC2]: fetch_metadata("requests") → latest version (per internal PEP 440
// sort via sort_versions_newest_first in the JSON / Simple API parsers) is classified
// as a release, not a pre-release (a/b/rc) or dev-release (.devN). Locks AC2 of the
// Phase-1.1 tracking issue (enhancement-pkg-mgr-phase-1-package-index-client-tracking).
//
// Uses fetch_metadata (JSON primary + Simple fallback) to exercise the JSON API path.
// The Simple API string-yanked path is exercised separately in IT-07.
#[tokio::test]
async fn it_06_fetch_metadata_requests_latest_is_release_not_prerelease() {
    let client = make_real_client();
    let meta = client
        .fetch_metadata("requests")
        .await
        .expect("IT-06: fetch_metadata(requests) must succeed");

    assert!(!meta.versions.is_empty(), "IT-06: versions list must be non-empty");
    let latest = &meta.versions[0]; // sorted newest-first by PEP 440 in simple_api.rs:53

    // AC2: assert latest is NOT a pre-release (a/b/rc + digits) and NOT a dev release (.devN).
    // Parse the last release segment: any trailing "aN" / "bN" / "rcN" / ".devN" disqualifies.
    let lower = latest.to_lowercase();
    let is_pre = {
        // look for a/b/rc immediately after a digit, followed by digits
        let bytes = lower.as_bytes();
        let mut i = 0;
        let mut found = false;
        while i < bytes.len() {
            let c = bytes[i];
            // match "aN" / "bN" / "rcN" suffix starts
            if (c == b'a' || c == b'b') && i > 0 && bytes[i - 1].is_ascii_digit()
                && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit()
            {
                found = true;
                break;
            }
            if c == b'r' && i + 1 < bytes.len() && bytes[i + 1] == b'c'
                && i > 0 && bytes[i - 1].is_ascii_digit()
                && i + 2 < bytes.len() && bytes[i + 2].is_ascii_digit()
            {
                found = true;
                break;
            }
            i += 1;
        }
        found
    };
    let is_dev = lower.contains(".dev");
    assert!(
        !is_pre,
        "IT-06/AC2: latest version `{}` must not be a pre-release (a/b/rc)",
        latest
    );
    assert!(
        !is_dev,
        "IT-06/AC2: latest version `{}` must not be a dev release (.devN)",
        latest
    );
}

// IT-07 [AC1/R1/R2]: fetch_metadata_simple("requests") against live pypi.org must return Ok
// and must include at least one file with a non-empty yanked_reason (string-yanked per PEP 691 §3.2).
// This confirms the YankedValue untagged enum fix: previously any string-yanked entry caused a
// ParseError that aborted the entire response; now all entries are parsed correctly.
#[tokio::test]
async fn it_07_fetch_metadata_simple_requests_handles_string_yanked() {
    let client = make_real_client();
    // REQ: R1 — must return Ok (not ParseError) even with string-yanked entries
    let meta = client
        .fetch_metadata_simple("requests")
        .await
        .expect("IT-07: fetch_metadata_simple(requests) must succeed despite string-yanked entries");

    assert!(!meta.versions.is_empty(), "IT-07: requests must have at least one version");
    assert_eq!(meta.source, "simple-api", "IT-07: source must be 'simple-api'");

    // REQ: R2 — string-yanked entries must set yanked=true AND populate yanked_reason
    let has_string_yanked = meta
        .releases
        .values()
        .flat_map(|files| files.iter())
        .any(|f| f.yanked && f.yanked_reason.as_deref().map(|r| !r.is_empty()).unwrap_or(false));

    assert!(
        has_string_yanked,
        "IT-07: at least one requests file must have yanked=true with a non-empty yanked_reason \
        (PyPI /simple/requests/ contains string-yanked entries; if this assertion fails the live \
        PyPI response may have changed — re-inspect the payload)"
    );
}
