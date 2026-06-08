// REQ: parse_simple_html — PEP 503 HTML anchor tag parsing → PackageMetadata
// REQ: parse_simple_json — PEP 691 JSON `files` array parsing → PackageMetadata
// REQ: Both fns are pure (no I/O, no async). Return IndexError::ParseError on malformed input.
// REQ: source = "simple-api" for all output PackageMetadata
// REQ: version extracted from filename (sdist: {name}-{ver}.tar.gz, wheel: {name}-{ver}-*.whl)
// REQ: versions sorted newest-first lex-desc (same placeholder as shard 2a)

use std::collections::BTreeMap;

use scraper::{Html, Selector};
use serde::Deserialize;

use crate::pkgmanage::pkgmgr::pep440::sort_versions_newest_first;
use crate::pkgmanage::pkgmgr::types::{FileHash, IndexError, PackageMetadata, ReleaseFile};

/// Extract the version string from a wheel or sdist filename.
///
/// Supports:
/// - sdist: `{name}-{version}.tar.gz`  → second `-`-separated component
/// - wheel: `{name}-{version}-{python}-{abi}-{platform}.whl`  → second component
///
/// Returns `None` when the filename does not match either pattern.
fn extract_version(filename: &str) -> Option<String> {
    let base = if filename.ends_with(".tar.gz") {
        filename.strip_suffix(".tar.gz")?
    } else if filename.ends_with(".whl") {
        filename.strip_suffix(".whl")?
    } else {
        return None;
    };
    // Components split by '-'; version is the second component (index 1).
    let mut parts = base.splitn(3, '-');
    let _name = parts.next()?;
    let version = parts.next()?;
    Some(version.to_string())
}

/// Build a `PackageMetadata` from a flat list of `ReleaseFile`s.
///
/// Groups files by version (parsed from filename), collects versions sorted
/// newest-first (lexicographic descending — PEP 440 upgrade deferred to shard 3),
/// and sets `source = "simple-api"`.
fn build_metadata(name: &str, files: Vec<ReleaseFile>) -> PackageMetadata {
    let mut releases: BTreeMap<String, Vec<ReleaseFile>> = BTreeMap::new();
    for file in files {
        if let Some(version) = extract_version(&file.filename) {
            releases.entry(version).or_default().push(file);
        }
        // Files whose version cannot be parsed are silently dropped.
    }

    let mut versions: Vec<String> = releases.keys().cloned().collect();
    sort_versions_newest_first(&mut versions);

    // Best-effort requires_python: take from the newest version's first file.
    let requires_python = versions
        .first()
        .and_then(|v| releases.get(v))
        .and_then(|fs| fs.first())
        .and_then(|f| f.requires_python.clone());

    PackageMetadata {
        name: name.to_string(),
        versions,
        releases,
        requires_python,
        source: "simple-api".to_string(),
    }
}

/// Strip the hash fragment from a URL.
///
/// PEP 503 encodes the hash as `#sha256=<hex>` appended to the URL.
/// This function splits at `#` and returns (clean_url, Some(FileHash)) or
/// (original_url, None) when no recognised fragment is present.
fn strip_hash_fragment(url: &str) -> (String, Option<FileHash>) {
    if let Some((base, fragment)) = url.split_once('#') {
        // Fragment format: `algorithm=digest`
        if let Some((algo, digest)) = fragment.split_once('=') {
            if matches!(algo, "sha256" | "sha384" | "sha512") {
                return (
                    base.to_string(),
                    Some(FileHash {
                        algorithm: algo.to_string(),
                        digest: digest.to_string(),
                    }),
                );
            }
        }
        // Unrecognised fragment — strip it but return no hash.
        (base.to_string(), None)
    } else {
        (url.to_string(), None)
    }
}

/// Parse a PEP 503 Simple API HTML response into [`PackageMetadata`].
///
/// Each `<a href="...">filename</a>` anchor yields one [`ReleaseFile`].
/// The hash is extracted from the URL fragment (`#sha256=<hex>`). Optional
/// attributes `data-requires-python`, `data-yanked`, and `data-dist-info-metadata`
/// are captured when present.
///
/// # Errors
///
/// Returns [`IndexError::ParseError`] when `body` is structurally empty (no `<a>`
/// tags found is *not* an error — that is a valid empty package index page).
/// The error is returned when the HTML itself cannot be parsed into a document
/// with the expected `<a>` anchor selector (which scraper makes infallible; so
/// ParseError is returned only when the CSS selector itself is invalid — which
/// is statically impossible here — or when caller-level malformed-input checks fail).
///
/// In practice this function only fails when `body` is completely unparseable
/// (e.g. not HTML at all and the scraper produces no document structure).
pub(crate) fn parse_simple_html(name: &str, body: &str) -> Result<PackageMetadata, IndexError> {
    let url_ctx = format!("simple-api:{name}");

    // scraper is infallible on parse (it is HTML5-lenient). We validate by
    // checking that we can build the anchor selector.
    let selector = Selector::parse("a").map_err(|e| IndexError::ParseError {
        url: url_ctx.clone(),
        detail: format!("CSS selector error: {e}"),
    })?;

    let document = Html::parse_document(body);

    // Reject input that doesn't look like HTML at all: if scraper finds zero
    // nodes (root element count == 0) after parsing. A well-formed empty index
    // page still has <html> / <body> structure.
    // We detect "completely non-HTML" input by checking for parse errors when
    // the body has no meaningful structure. The simplest sentinel: if the body
    // doesn't contain `<` at all it cannot be HTML.
    if !body.contains('<') {
        return Err(IndexError::ParseError {
            url: url_ctx,
            detail: "body contains no HTML elements".to_string(),
        });
    }

    let mut files: Vec<ReleaseFile> = Vec::new();

    for anchor in document.select(&selector) {
        let href = match anchor.value().attr("href") {
            Some(h) => h,
            None => continue, // anchor without href — skip
        };

        // The inner text of the anchor is the filename.
        let filename: String = anchor.text().collect::<String>().trim().to_string();
        if filename.is_empty() {
            continue;
        }

        let (clean_url, hash_opt) = strip_hash_fragment(href);

        // Fall back to empty sentinel hash when none in fragment.
        let hash = hash_opt.unwrap_or_else(FileHash::default);

        let requires_python = anchor
            .value()
            .attr("data-requires-python")
            .map(|s| s.to_string());

        // PEP 503: data-yanked attribute presence means yanked=true.
        // The attribute VALUE, when non-empty, is the reason string (PEP 592).
        let (yanked, yanked_reason) = match anchor.value().attr("data-yanked") {
            None => (false, None),
            Some("") => (true, None),
            Some(reason) => (true, Some(reason.to_string())),
        };

        // `data-dist-info-metadata` can be a hash like "sha256=<hex>" or just
        // the empty attribute (presence means true). Map to a serde_json::Value.
        let dist_info_metadata: serde_json::Value =
            match anchor.value().attr("data-dist-info-metadata") {
                None => serde_json::Value::Null,
                Some("") | Some("true") => serde_json::Value::Bool(true),
                Some(val) => {
                    // Try to parse as `algo=digest` → {"algo": "digest"}
                    if let Some((algo, digest)) = val.split_once('=') {
                        serde_json::json!({ algo: digest })
                    } else {
                        serde_json::Value::Bool(true)
                    }
                }
            };

        files.push(ReleaseFile {
            filename,
            url: clean_url,
            hash,
            requires_python,
            size: None,
            upload_time: None,
            yanked,
            yanked_reason,
            dist_info_metadata,
            source: Some("simple-api".to_string()),
        });
    }

    Ok(build_metadata(name, files))
}

/// PEP 691 §3.2 allows `yanked` to be a boolean OR a string (reason text).
/// PEP 592: a string value means yanked=true with that string as the reason.
/// `#[serde(untagged)]` makes serde try each variant in declaration order.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum YankedValue {
    Bool(bool),
    Reason(String),
}

/// Internal serde shape for a single entry in the PEP 691 `files` array.
#[derive(Debug, Deserialize)]
struct Pep691File {
    filename: String,
    url: String,
    #[serde(default)]
    hashes: Pep691Hashes,
    #[serde(rename = "requires-python", default)]
    requires_python: Option<String>,
    /// `yanked` may be absent (→ false), bool, or a reason string per PEP 691 §3.2.
    #[serde(default)]
    yanked: Option<YankedValue>,
    #[serde(rename = "dist-info-metadata", default)]
    dist_info_metadata: serde_json::Value,
}

#[derive(Debug, Default, Deserialize)]
struct Pep691Hashes {
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    sha384: Option<String>,
    #[serde(default)]
    sha512: Option<String>,
}

impl Pep691Hashes {
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

/// Top-level shape for a PEP 691 Simple API JSON response.
#[derive(Debug, Deserialize)]
struct Pep691Response {
    files: Vec<Pep691File>,
}

/// Parse a PEP 691 Simple API JSON response body into [`PackageMetadata`].
///
/// Expects the top-level `{ "files": [...] }` structure. Each entry in
/// `files` maps to one [`ReleaseFile`]. The URL is stripped of its hash
/// fragment (same as HTML path), and `hashes.sha256` is used as the primary
/// hash.
///
/// # Errors
///
/// Returns [`IndexError::ParseError`] when `body` is not valid JSON or lacks
/// the expected `files` key.
pub(crate) fn parse_simple_json(name: &str, body: &str) -> Result<PackageMetadata, IndexError> {
    let url_ctx = format!("simple-api:{name}");

    let raw: Pep691Response = serde_json::from_str(body).map_err(|e| IndexError::ParseError {
        url: url_ctx.clone(),
        detail: format!("JSON parse error: {e}"),
    })?;

    let files: Vec<ReleaseFile> = raw
        .files
        .into_iter()
        .map(|f| {
            let (clean_url, _frag_hash) = strip_hash_fragment(&f.url);
            // Prefer explicit hashes map over fragment hash.
            let hash = f.hashes.best();
            // PEP 691 §3.2 / PEP 592: flatten YankedValue to (bool, Option<String>).
            //   None                → yanked=false, yanked_reason=None
            //   Some(Bool(b))       → yanked=b,     yanked_reason=None
            //   Some(Reason(s))     → yanked=true,  yanked_reason=Some(s) (string presence ≡ yanked)
            let (yanked, yanked_reason) = match f.yanked {
                None => (false, None),
                Some(YankedValue::Bool(b)) => (b, None),
                Some(YankedValue::Reason(s)) => (true, Some(s)),
            };
            ReleaseFile {
                filename: f.filename,
                url: clean_url,
                hash,
                requires_python: f.requires_python,
                size: None,
                upload_time: None,
                yanked,
                yanked_reason,
                dist_info_metadata: f.dist_info_metadata,
                source: Some("simple-api".to_string()),
            }
        })
        .collect();

    Ok(build_metadata(name, files))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: test_parse_simple_html_basic — one <a> tag with sha256 fragment
    #[test]
    fn test_parse_simple_html_basic() {
        let body = r#"<!DOCTYPE html>
<html>
<body>
<a href="https://files.example.com/requests-2.31.0-py3-none-any.whl#sha256=abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
   data-requires-python=">=3.7">requests-2.31.0-py3-none-any.whl</a>
</body>
</html>"#;

        let meta = parse_simple_html("requests", body).expect("should parse successfully");

        assert_eq!(meta.name, "requests");
        assert_eq!(meta.source, "simple-api");
        assert_eq!(meta.versions.len(), 1);
        assert_eq!(meta.versions[0], "2.31.0");

        let files = meta
            .releases
            .get("2.31.0")
            .expect("version 2.31.0 should exist");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "requests-2.31.0-py3-none-any.whl");
        assert_eq!(files[0].hash.algorithm, "sha256");
        assert_eq!(
            files[0].hash.digest,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        );
        // URL should NOT contain the fragment
        assert!(!files[0].url.contains('#'));
        assert_eq!(
            files[0].url,
            "https://files.example.com/requests-2.31.0-py3-none-any.whl"
        );
        assert_eq!(files[0].requires_python.as_deref(), Some(">=3.7"));
        assert!(!files[0].yanked);
        assert_eq!(files[0].source.as_deref(), Some("simple-api"));
    }

    // REQ: test_parse_simple_html_malformed → IndexError::ParseError
    #[test]
    fn test_parse_simple_html_malformed() {
        // Plain text with no HTML elements at all.
        let body = "this is not html, no angle brackets";
        let err = parse_simple_html("mypkg", body).expect_err("should fail on non-HTML");
        assert!(
            matches!(err, IndexError::ParseError { .. }),
            "expected IndexError::ParseError, got: {:?}",
            err
        );
    }

    // REQ: test_parse_simple_html_yanked — data-yanked attribute sets yanked=true
    #[test]
    fn test_parse_simple_html_yanked_attr() {
        let body = r#"<html><body>
<a href="https://files.example.com/mypkg-1.0.0.tar.gz#sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   data-yanked="">mypkg-1.0.0.tar.gz</a>
</body></html>"#;
        let meta = parse_simple_html("mypkg", body).unwrap();
        let files = meta.releases.get("1.0.0").unwrap();
        assert!(files[0].yanked, "yanked attr should set yanked=true");
    }

    // REQ: test_parse_simple_html_multiple_files — grouping by version works
    #[test]
    fn test_parse_simple_html_multiple_files() {
        let body = r#"<html><body>
<a href="https://files.example.com/pkg-1.0.0-py3-none-any.whl#sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa">pkg-1.0.0-py3-none-any.whl</a>
<a href="https://files.example.com/pkg-1.0.0.tar.gz#sha256=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb">pkg-1.0.0.tar.gz</a>
<a href="https://files.example.com/pkg-0.9.0-py3-none-any.whl#sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc">pkg-0.9.0-py3-none-any.whl</a>
</body></html>"#;
        let meta = parse_simple_html("pkg", body).unwrap();
        // 2 distinct versions
        assert_eq!(meta.versions.len(), 2);
        // Newest first (lex desc): 1.0.0 > 0.9.0
        assert_eq!(meta.versions[0], "1.0.0");
        assert_eq!(meta.versions[1], "0.9.0");
        // 1.0.0 has 2 files (whl + tar.gz)
        assert_eq!(meta.releases["1.0.0"].len(), 2);
        // 0.9.0 has 1 file
        assert_eq!(meta.releases["0.9.0"].len(), 1);
    }

    // REQ: test_parse_simple_json_basic — PEP 691 JSON with 2 files
    #[test]
    fn test_parse_simple_json_basic() {
        let body = r#"{
            "meta": { "api-version": "1.0" },
            "name": "requests",
            "files": [
                {
                    "filename": "requests-2.31.0-py3-none-any.whl",
                    "url": "https://files.example.com/requests-2.31.0-py3-none-any.whl",
                    "hashes": { "sha256": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890" },
                    "requires-python": ">=3.7",
                    "yanked": false,
                    "dist-info-metadata": false
                },
                {
                    "filename": "requests-2.31.0.tar.gz",
                    "url": "https://files.example.com/requests-2.31.0.tar.gz",
                    "hashes": { "sha256": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef" },
                    "requires-python": ">=3.7",
                    "yanked": false
                }
            ]
        }"#;

        let meta = parse_simple_json("requests", body).expect("should parse successfully");

        assert_eq!(meta.name, "requests");
        assert_eq!(meta.source, "simple-api");
        assert_eq!(meta.versions.len(), 1);
        assert_eq!(meta.versions[0], "2.31.0");

        let files = meta
            .releases
            .get("2.31.0")
            .expect("version 2.31.0 should exist");
        assert_eq!(files.len(), 2, "both wheel and sdist should be present");

        // Verify hashes
        assert!(files.iter().all(|f| f.hash.algorithm == "sha256"));
        assert!(files
            .iter()
            .all(|f| f.source.as_deref() == Some("simple-api")));
    }

    // REQ: test_parse_simple_json_malformed — invalid JSON → IndexError::ParseError
    #[test]
    fn test_parse_simple_json_malformed() {
        let err = parse_simple_json("mypkg", "not-json{{{{").expect_err("should fail on bad JSON");
        assert!(
            matches!(err, IndexError::ParseError { .. }),
            "expected IndexError::ParseError, got: {:?}",
            err
        );
    }

    // REQ: test_parse_simple_json_missing_files_key — missing `files` key → ParseError
    #[test]
    fn test_parse_simple_json_missing_files_key() {
        // JSON without required `files` key
        let err = parse_simple_json("mypkg", r#"{"meta": {"api-version": "1.0"}}"#)
            .expect_err("should fail with missing files key");
        assert!(
            matches!(err, IndexError::ParseError { .. }),
            "expected IndexError::ParseError, got: {:?}",
            err
        );
    }

    // REQ: test_strip_hash_fragment — URL splitting logic
    #[test]
    fn test_strip_hash_fragment() {
        let (url, hash) = strip_hash_fragment(
            "https://files.example.com/pkg-1.0.0.whl#sha256=deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        );
        assert_eq!(url, "https://files.example.com/pkg-1.0.0.whl");
        let h = hash.expect("should have hash");
        assert_eq!(h.algorithm, "sha256");
        assert_eq!(
            h.digest,
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        );

        // No fragment
        let (url2, hash2) = strip_hash_fragment("https://files.example.com/pkg-1.0.0.whl");
        assert_eq!(url2, "https://files.example.com/pkg-1.0.0.whl");
        assert!(hash2.is_none());
    }

    // REQ: AC3 [R3] — HTML parser: non-empty data-yanked value populates yanked_reason
    #[test]
    fn test_parse_simple_html_yanked_attr_with_reason() {
        // Empty data-yanked → yanked=true, yanked_reason=None
        let body_empty = r#"<html><body>
<a href="https://files.example.com/mypkg-1.0.0.tar.gz#sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   data-yanked="">mypkg-1.0.0.tar.gz</a>
</body></html>"#;
        let meta = parse_simple_html("mypkg", body_empty).unwrap();
        let files = meta.releases.get("1.0.0").unwrap();
        assert!(files[0].yanked, "empty data-yanked must set yanked=true");
        assert_eq!(
            files[0].yanked_reason, None,
            "empty data-yanked must leave yanked_reason=None"
        );

        // Non-empty data-yanked → yanked=true, yanked_reason=Some(reason)
        let body_reason = r#"<html><body>
<a href="https://files.example.com/mypkg-2.0.0.tar.gz#sha256=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
   data-yanked="reason here">mypkg-2.0.0.tar.gz</a>
</body></html>"#;
        let meta2 = parse_simple_html("mypkg", body_reason).unwrap();
        let files2 = meta2.releases.get("2.0.0").unwrap();
        assert!(
            files2[0].yanked,
            "non-empty data-yanked must set yanked=true"
        );
        assert_eq!(
            files2[0].yanked_reason.as_deref(),
            Some("reason here"),
            "non-empty data-yanked value must populate yanked_reason"
        );

        // Absent data-yanked → yanked=false, yanked_reason=None
        let body_absent = r#"<html><body>
<a href="https://files.example.com/mypkg-3.0.0.tar.gz#sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc">mypkg-3.0.0.tar.gz</a>
</body></html>"#;
        let meta3 = parse_simple_html("mypkg", body_absent).unwrap();
        let files3 = meta3.releases.get("3.0.0").unwrap();
        assert!(
            !files3[0].yanked,
            "absent data-yanked must leave yanked=false"
        );
        assert_eq!(
            files3[0].yanked_reason, None,
            "absent data-yanked must leave yanked_reason=None"
        );
    }

    // REQ: AC2 [R5] — JSON parser: fixture with bool-yanked AND string-yanked entries; assert correct split
    #[test]
    fn test_parse_pep691_json_with_string_yanked_reason() {
        let body = r#"{
            "meta": { "api-version": "1.0" },
            "name": "requests",
            "files": [
                {
                    "filename": "requests-2.28.0-py3-none-any.whl",
                    "url": "https://files.example.com/requests-2.28.0-py3-none-any.whl",
                    "hashes": { "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
                    "yanked": true
                },
                {
                    "filename": "requests-2.27.0-py3-none-any.whl",
                    "url": "https://files.example.com/requests-2.27.0-py3-none-any.whl",
                    "hashes": { "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" },
                    "yanked": "Yanked due to conflicts with CVE-2024-35195 mitigation"
                },
                {
                    "filename": "requests-2.26.0-py3-none-any.whl",
                    "url": "https://files.example.com/requests-2.26.0-py3-none-any.whl",
                    "hashes": { "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" },
                    "yanked": false
                },
                {
                    "filename": "requests-2.25.0-py3-none-any.whl",
                    "url": "https://files.example.com/requests-2.25.0-py3-none-any.whl",
                    "hashes": { "sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd" }
                }
            ]
        }"#;

        let meta = parse_simple_json("requests", body)
            .expect("must parse successfully with string-yanked entries");

        // Bool yanked=true → yanked=true, yanked_reason=None
        let v2280 = &meta.releases["2.28.0"][0];
        assert!(
            v2280.yanked,
            "2.28.0: bool yanked=true must set yanked=true"
        );
        assert_eq!(
            v2280.yanked_reason, None,
            "2.28.0: bool yanked=true must leave yanked_reason=None"
        );

        // String yanked → yanked=true, yanked_reason=Some(reason)
        let v2270 = &meta.releases["2.27.0"][0];
        assert!(v2270.yanked, "2.27.0: string yanked must set yanked=true");
        assert_eq!(
            v2270.yanked_reason.as_deref(),
            Some("Yanked due to conflicts with CVE-2024-35195 mitigation"),
            "2.27.0: string yanked must populate yanked_reason"
        );

        // Bool yanked=false → yanked=false, yanked_reason=None
        let v2260 = &meta.releases["2.26.0"][0];
        assert!(
            !v2260.yanked,
            "2.26.0: bool yanked=false must set yanked=false"
        );
        assert_eq!(
            v2260.yanked_reason, None,
            "2.26.0: bool yanked=false must leave yanked_reason=None"
        );

        // Absent yanked → yanked=false, yanked_reason=None
        let v2250 = &meta.releases["2.25.0"][0];
        assert!(
            !v2250.yanked,
            "2.25.0: absent yanked must default to yanked=false"
        );
        assert_eq!(
            v2250.yanked_reason, None,
            "2.25.0: absent yanked must leave yanked_reason=None"
        );
    }

    // REQ: tick-112 test-coverage — strip_hash_fragment alt-algo + malformed-fragment branches
    #[test]
    fn test_strip_hash_fragment_alt_algos_and_malformed() {
        let (u, h) = strip_hash_fragment("https://x/p.whl#sha384=d384");
        assert_eq!(u, "https://x/p.whl");
        assert_eq!(h.unwrap().algorithm, "sha384");

        let (u, h) = strip_hash_fragment("https://x/p.whl#sha512=d512");
        assert_eq!(u, "https://x/p.whl");
        assert_eq!(h.unwrap().algorithm, "sha512");

        // Unrecognised algo — URL stripped, hash dropped
        let (u, h) = strip_hash_fragment("https://x/p.whl#md5=abcd");
        assert_eq!(u, "https://x/p.whl");
        assert!(h.is_none(), "non-sha algo must not produce a hash");

        // Fragment without '=' — URL stripped, hash dropped
        let (u, h) = strip_hash_fragment("https://x/p.whl#nohash");
        assert_eq!(u, "https://x/p.whl");
        assert!(h.is_none(), "fragment without '=' must not produce a hash");
    }
}
