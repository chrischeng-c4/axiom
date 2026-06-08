// REQ: parse_json_metadata — pure function, maps PyPI JSON response → PackageMetadata
// REQ: Mapping: releases[v][i].digests.sha256 → ReleaseFile.hash (algorithm = "sha256")
// REQ: versions = all keys of JSON releases object, sorted newest-first (lexicographic desc)

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::pkgmanage::pkgmgr::pep440::sort_versions_newest_first;
use crate::pkgmanage::pkgmgr::types::{FileHash, IndexError, PackageMetadata, ReleaseFile};

/// Internal serde shape for the PyPI JSON endpoint `GET /pypi/{name}/json`.
#[derive(Debug, Deserialize)]
struct PypiResponse {
    info: PypiInfo,
    releases: BTreeMap<String, Vec<PypiReleaseFile>>,
}

#[derive(Debug, Deserialize)]
struct PypiInfo {
    name: String,
    #[serde(default)]
    requires_python: Option<String>,
}

/// Raw release file record as returned by the PyPI JSON API.
#[derive(Debug, Deserialize)]
struct PypiReleaseFile {
    filename: String,
    url: String,
    /// May be absent on very old package entries; we default to an empty map.
    #[serde(default)]
    digests: PypiDigests,
    #[serde(default)]
    requires_python: Option<String>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    upload_time: Option<String>,
    #[serde(default)]
    yanked: bool,
    #[serde(default)]
    yanked_reason: Option<String>,
    /// PEP 658 — may be false, true, or {"sha256": "..."}.
    #[serde(default)]
    dist_info_metadata: serde_json::Value,
}

/// The `digests` sub-object in a PyPI release file record.
///
/// When the `digests` field is absent entirely (very old entries), all fields
/// default to `None` / empty string, and `FileHash { algorithm: "", digest: "" }`
/// is produced — a sentinel that callers can detect.
#[derive(Debug, Default, Deserialize)]
struct PypiDigests {
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    sha384: Option<String>,
    #[serde(default)]
    sha512: Option<String>,
}

impl PypiDigests {
    /// Returns the best available hash, preferring sha256 > sha384 > sha512.
    ///
    /// If none are present, returns an empty `FileHash` sentinel
    /// (`algorithm = ""`, `digest = ""`). Callers that require a valid hash
    /// should treat this sentinel as "no hash available".
    fn best(&self) -> FileHash {
        if let Some(d) = &self.sha256 {
            return FileHash {
                algorithm: "sha256".into(),
                digest: d.clone(),
            };
        }
        if let Some(d) = &self.sha384 {
            return FileHash {
                algorithm: "sha384".into(),
                digest: d.clone(),
            };
        }
        if let Some(d) = &self.sha512 {
            return FileHash {
                algorithm: "sha512".into(),
                digest: d.clone(),
            };
        }
        FileHash::default()
    }
}

/// Parse a raw PyPI JSON API response body into [`PackageMetadata`].
///
/// # Errors
///
/// Returns [`IndexError::ParseError`] when the input is not valid JSON or does not
/// conform to the expected PyPI response shape.
///
/// # Example
///
/// ```rust
/// # use mamba::pkgmanage::pkgmgr::json_api::parse_json_metadata;
/// let body = r#"{
///   "info": { "name": "example", "version": "1.0.0", "requires_python": ">=3.8" },
///   "releases": {}
/// }"#;
/// let meta = parse_json_metadata(body).unwrap();
/// assert_eq!(meta.name, "example");
/// assert!(meta.versions.is_empty());
/// ```
pub fn parse_json_metadata(body: &str) -> Result<PackageMetadata, IndexError> {
    let raw: PypiResponse = serde_json::from_str(body).map_err(|e| IndexError::ParseError {
        url: String::new(),
        detail: e.to_string(),
    })?;

    let releases: BTreeMap<String, Vec<ReleaseFile>> = raw
        .releases
        .into_iter()
        .map(|(version, files)| {
            let release_files = files
                .into_iter()
                .map(|f| ReleaseFile {
                    filename: f.filename,
                    url: f.url,
                    hash: f.digests.best(),
                    requires_python: f.requires_python,
                    size: f.size,
                    upload_time: f.upload_time,
                    yanked: f.yanked,
                    yanked_reason: f.yanked_reason,
                    dist_info_metadata: f.dist_info_metadata,
                    source: Some("json-api".into()),
                })
                .collect();
            (version, release_files)
        })
        .collect();

    // Collect versions sorted newest-first using PEP 440 ordering (shard 3b).
    let mut versions: Vec<String> = releases.keys().cloned().collect();
    sort_versions_newest_first(&mut versions);

    Ok(PackageMetadata {
        name: raw.info.name,
        versions,
        releases,
        requires_python: raw.info.requires_python,
        source: "json-api".into(),
    })
}

/// Internal serde shape for the per-version PyPI JSON endpoint
/// `GET /pypi/{name}/{version}/json`. Only carries the fields the resolver
/// needs today; everything else falls through unread.
#[derive(Debug, Deserialize)]
struct PypiVersionResponse {
    info: PypiVersionInfo,
}

#[derive(Debug, Deserialize)]
struct PypiVersionInfo {
    #[serde(default)]
    requires_dist: Option<Vec<String>>,
}

/// Parse the body of `GET /pypi/{name}/{version}/json` and return the
/// raw PEP 508 `requires_dist` strings exposed by the index. Returns an
/// empty Vec when the field is absent or null.
pub fn parse_version_requires(body: &str) -> Result<Vec<String>, IndexError> {
    let raw: PypiVersionResponse =
        serde_json::from_str(body).map_err(|e| IndexError::ParseError {
            url: String::new(),
            detail: e.to_string(),
        })?;
    Ok(raw.info.requires_dist.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: test_parse_minimal_metadata — 1 release, 1 wheel, asserts name/versions/releases/digest
    #[test]
    fn test_parse_minimal_metadata() {
        let body = r#"{
            "info": {
                "name": "requests",
                "version": "2.31.0",
                "requires_python": ">=3.7"
            },
            "releases": {
                "2.31.0": [
                    {
                        "filename": "requests-2.31.0-py3-none-any.whl",
                        "url": "https://files.pythonhosted.org/packages/requests-2.31.0-py3-none-any.whl",
                        "digests": {
                            "sha256": "58cd2187423839aa6e34d77a8f45b4a28a5f3e0e8c7e6b7b0e7e8e8e8e8e8e8e"
                        },
                        "requires_python": ">=3.7",
                        "yanked": false
                    }
                ]
            }
        }"#;

        let meta = parse_json_metadata(body).expect("should parse successfully");
        assert_eq!(meta.name, "requests");
        assert_eq!(meta.versions, vec!["2.31.0"]);
        assert!(meta.releases.contains_key("2.31.0"));
        let files = &meta.releases["2.31.0"];
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].hash.algorithm, "sha256");
        // digest should be a non-empty hex string
        assert!(!files[0].hash.digest.is_empty());
        assert_eq!(files[0].hash.digest.len(), 64);
        assert_eq!(meta.source, "json-api");
    }

    // REQ: test_parse_missing_hashes_defaults_to_empty — absent digests field produces empty sentinel
    #[test]
    fn test_parse_missing_hashes_defaults_to_empty() {
        // A release file with no `digests` key at all — very old PyPI entries.
        let body = r#"{
            "info": {
                "name": "oldpkg",
                "version": "0.1.0"
            },
            "releases": {
                "0.1.0": [
                    {
                        "filename": "oldpkg-0.1.0.tar.gz",
                        "url": "https://files.pythonhosted.org/packages/oldpkg-0.1.0.tar.gz"
                    }
                ]
            }
        }"#;

        let meta =
            parse_json_metadata(body).expect("should parse successfully even without digests");
        let files = &meta.releases["0.1.0"];
        assert_eq!(files.len(), 1);
        // Sentinel: both algorithm and digest are empty strings
        assert_eq!(files[0].hash.algorithm, "");
        assert_eq!(files[0].hash.digest, "");
    }

    // REQ: test_parse_invalid_json_returns_parse_error — non-JSON input → IndexError::ParseError
    #[test]
    fn test_parse_invalid_json_returns_parse_error() {
        let err = parse_json_metadata("not-json").expect_err("should fail on invalid JSON");
        assert!(
            matches!(err, IndexError::ParseError { .. }),
            "expected IndexError::ParseError, got: {:?}",
            err
        );
    }

    #[test]
    fn test_parse_empty_releases_map() {
        let body = r#"{
            "info": { "name": "emptypkg", "version": "0.0.1" },
            "releases": {}
        }"#;
        let meta = parse_json_metadata(body).unwrap();
        assert_eq!(meta.name, "emptypkg");
        assert!(meta.versions.is_empty());
        assert!(meta.releases.is_empty());
        assert!(meta.requires_python.is_none());
    }

    #[test]
    fn test_parse_multiple_versions_and_files() {
        let body = r#"{
            "info": { "name": "mypkg", "version": "1.2.0", "requires_python": ">=3.9" },
            "releases": {
                "1.1.0": [
                    {
                        "filename": "mypkg-1.1.0-py3-none-any.whl",
                        "url": "https://example.com/mypkg-1.1.0-py3-none-any.whl",
                        "digests": { "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" }
                    }
                ],
                "1.2.0": [
                    {
                        "filename": "mypkg-1.2.0-py3-none-any.whl",
                        "url": "https://example.com/mypkg-1.2.0-py3-none-any.whl",
                        "digests": { "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" }
                    },
                    {
                        "filename": "mypkg-1.2.0.tar.gz",
                        "url": "https://example.com/mypkg-1.2.0.tar.gz",
                        "digests": { "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" }
                    }
                ]
            }
        }"#;
        let meta = parse_json_metadata(body).unwrap();
        // versions sorted newest-first (lexicographic descending)
        assert_eq!(meta.versions, vec!["1.2.0", "1.1.0"]);
        assert_eq!(meta.releases.len(), 2);
        assert_eq!(meta.releases["1.2.0"].len(), 2);
        assert_eq!(meta.releases["1.1.0"].len(), 1);
        assert_eq!(meta.requires_python.as_deref(), Some(">=3.9"));
    }

    // REQ: tick-108 test-coverage — PypiDigests::best() priority sha256 > sha384 > sha512
    #[test]
    fn test_digest_priority_sha384_and_sha512_fallback() {
        let only_sha384 = r#"{
            "info": { "name": "p", "version": "1.0.0" },
            "releases": { "1.0.0": [ { "filename": "p-1.0.0.whl", "url": "u",
                "digests": { "sha384": "d384" } } ] }
        }"#;
        let m = parse_json_metadata(only_sha384).unwrap();
        assert_eq!(m.releases["1.0.0"][0].hash.algorithm, "sha384");
        assert_eq!(m.releases["1.0.0"][0].hash.digest, "d384");

        let only_sha512 = r#"{
            "info": { "name": "p", "version": "1.0.0" },
            "releases": { "1.0.0": [ { "filename": "p-1.0.0.whl", "url": "u",
                "digests": { "sha512": "d512" } } ] }
        }"#;
        let m = parse_json_metadata(only_sha512).unwrap();
        assert_eq!(m.releases["1.0.0"][0].hash.algorithm, "sha512");

        let all_three = r#"{
            "info": { "name": "p", "version": "1.0.0" },
            "releases": { "1.0.0": [ { "filename": "p-1.0.0.whl", "url": "u",
                "digests": { "sha256": "d256", "sha384": "d384", "sha512": "d512" } } ] }
        }"#;
        let m = parse_json_metadata(all_three).unwrap();
        assert_eq!(
            m.releases["1.0.0"][0].hash.algorithm, "sha256",
            "sha256 must win when all three present"
        );
    }
}
