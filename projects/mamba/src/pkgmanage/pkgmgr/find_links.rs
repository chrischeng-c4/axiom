// `--find-links` artifact filename classifier (Tick 124).
//
// pip's `--find-links` flag (uv supports it identically) lets the user
// point the resolver at a directory OR an HTML listing page containing
// already-downloaded wheels and sdists. The flag itself is parsed by
// `requirements_options.rs`; this module covers the next layer down:
// given a filename, classify it as a wheel or sdist candidate and
// extract the PEP 503-normalized name + PEP 440 version so the
// resolver can dispatch against it.
//
// Supported artifact filenames:
//   * `<name>-<version>(-<build_tag>)?-<python>-<abi>-<platform>.whl`
//       — PEP 427 wheels, delegated to `wheel_filename::WheelFilename`.
//   * `<name>-<version>.tar.gz`           — PEP 625 source distribution.
//   * `<name>-<version>.zip`              — legacy source distribution.
//
// I/O is intentionally out of scope: this module classifies filenames
// only. Filesystem enumeration and HTTP listing-page scraping live in
// dedicated callers (the `simple_api` + `requirements_loader` modules
// already cover those transports).

use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::types::IndexError;
use crate::pkgmanage::pkgmgr::wheel_filename::parse_wheel_filename;
use std::collections::BTreeMap;

const FIND_LINKS_DETAIL: &str = "<find-links artifact>";

/// Kind of artifact a `find-links` filename advertises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    Wheel,
    Sdist,
}

impl ArtifactKind {
    /// Stable lower-case discriminant for use in lockfile/source-kind
    /// emission.
    pub fn as_str(self) -> &'static str {
        match self {
            ArtifactKind::Wheel => "wheel",
            ArtifactKind::Sdist => "sdist",
        }
    }
}

/// Classified `find-links` entry. The `name` field is the
/// PEP 503-normalized form so resolver lookups are case/-/_-insensitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindLinksEntry {
    /// PEP 503-normalized project name.
    pub name: String,
    /// Verbatim project name as it appears in the filename (before
    /// PEP 503 normalization).
    pub raw_name: String,
    /// Verbatim version string. Callers needing structural compare
    /// should route through `pep440::parse`.
    pub version: String,
    /// Original filename, including extension.
    pub filename: String,
    /// Wheel vs sdist.
    pub kind: ArtifactKind,
}

/// Classify a single artifact filename. Returns `Ok(None)` when the
/// extension is unrecognized — callers should treat that as "skip,
/// not a wheel or sdist" rather than an error. Returns `Err` only
/// when the file *looks* like a known kind but is malformed
/// (e.g. `.whl` with bad tag layout, `.tar.gz` with no version).
pub fn classify_filename(filename: &str) -> Result<Option<FindLinksEntry>, IndexError> {
    if filename.ends_with(".whl") {
        let parsed = parse_wheel_filename(filename)?;
        return Ok(Some(FindLinksEntry {
            name: pep503_normalize(&parsed.distribution),
            raw_name: parsed.distribution,
            version: parsed.version,
            filename: filename.to_string(),
            kind: ArtifactKind::Wheel,
        }));
    }

    if let Some(stem) = strip_sdist_suffix(filename) {
        let (raw_name, version) = split_sdist_stem(stem)?;
        return Ok(Some(FindLinksEntry {
            name: pep503_normalize(raw_name),
            raw_name: raw_name.to_string(),
            version: version.to_string(),
            filename: filename.to_string(),
            kind: ArtifactKind::Sdist,
        }));
    }

    Ok(None)
}

/// Classify a batch of filenames. Filenames that classify to `None`
/// (unknown extension) are silently dropped. The first hard error
/// (malformed wheel name, sdist with no dash) is surfaced to the
/// caller.
pub fn classify_many<I, S>(filenames: I) -> Result<Vec<FindLinksEntry>, IndexError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = Vec::new();
    for f in filenames {
        if let Some(entry) = classify_filename(f.as_ref())? {
            out.push(entry);
        }
    }
    Ok(out)
}

/// Group classified entries by PEP 503-normalized package name. Input
/// order is preserved within each bucket. Bucket order is alphabetical
/// (BTreeMap), matching the order the resolver expects so lockfile
/// diffs stay stable.
pub fn group_by_name(entries: Vec<FindLinksEntry>) -> BTreeMap<String, Vec<FindLinksEntry>> {
    let mut grouped: BTreeMap<String, Vec<FindLinksEntry>> = BTreeMap::new();
    for entry in entries {
        grouped.entry(entry.name.clone()).or_default().push(entry);
    }
    grouped
}

/// Strip the sdist-supported suffix from `filename`. PEP 625 makes
/// `.tar.gz` the only valid future sdist extension, but pip continues
/// to recognize `.zip` for backwards compatibility, as does uv.
fn strip_sdist_suffix(filename: &str) -> Option<&str> {
    filename
        .strip_suffix(".tar.gz")
        .or_else(|| filename.strip_suffix(".zip"))
}

/// Split `<name>-<version>` at the LAST `-`. Both sides must be
/// non-empty; the version may itself contain hyphens (in PEP 440
/// local-version labels like `1.0.0-cu118`) — but PEP 625 dictates
/// that the sdist filename uses `.` separators for local versions
/// (e.g. `1.0.0.cu118`), so we can split safely at the last hyphen.
fn split_sdist_stem(stem: &str) -> Result<(&str, &str), IndexError> {
    let Some(dash) = stem.rfind('-') else {
        return Err(IndexError::ParseError {
            url: FIND_LINKS_DETAIL.into(),
            detail: format!("sdist filename {stem:?} has no `-` separator"),
        });
    };
    let name = &stem[..dash];
    let version = &stem[dash + 1..];
    if name.is_empty() {
        return Err(IndexError::ParseError {
            url: FIND_LINKS_DETAIL.into(),
            detail: format!("sdist filename {stem:?} has empty name part"),
        });
    }
    if version.is_empty() {
        return Err(IndexError::ParseError {
            url: FIND_LINKS_DETAIL.into(),
            detail: format!("sdist filename {stem:?} has empty version part"),
        });
    }
    Ok((name, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_pure_python_wheel() {
        let e = classify_filename("requests-2.31.0-py3-none-any.whl")
            .unwrap()
            .unwrap();
        assert_eq!(e.name, "requests");
        assert_eq!(e.raw_name, "requests");
        assert_eq!(e.version, "2.31.0");
        assert_eq!(e.kind, ArtifactKind::Wheel);
        assert_eq!(e.filename, "requests-2.31.0-py3-none-any.whl");
    }

    #[test]
    fn classifies_pep625_sdist() {
        let e = classify_filename("requests-2.31.0.tar.gz")
            .unwrap()
            .unwrap();
        assert_eq!(e.name, "requests");
        assert_eq!(e.raw_name, "requests");
        assert_eq!(e.version, "2.31.0");
        assert_eq!(e.kind, ArtifactKind::Sdist);
    }

    #[test]
    fn classifies_legacy_zip_sdist() {
        let e = classify_filename("setuptools-68.2.0.zip").unwrap().unwrap();
        assert_eq!(e.name, "setuptools");
        assert_eq!(e.kind, ArtifactKind::Sdist);
    }

    #[test]
    fn normalizes_name_to_pep503() {
        let w = classify_filename("Zope_Interface-5.0.0-py3-none-any.whl")
            .unwrap()
            .unwrap();
        assert_eq!(w.name, "zope-interface");
        assert_eq!(w.raw_name, "Zope_Interface");

        let s = classify_filename("Zope.Interface-5.0.0.tar.gz")
            .unwrap()
            .unwrap();
        assert_eq!(s.name, "zope-interface");
        assert_eq!(s.raw_name, "Zope.Interface");
    }

    #[test]
    fn unknown_extension_returns_none() {
        assert!(classify_filename("README.md").unwrap().is_none());
        assert!(classify_filename("requests-2.31.0.egg").unwrap().is_none());
        assert!(classify_filename("requests-2.31.0.tar.bz2")
            .unwrap()
            .is_none());
        assert!(classify_filename("not-an-artifact").unwrap().is_none());
    }

    #[test]
    fn malformed_sdist_no_dash_errors() {
        assert!(classify_filename("noversion.tar.gz").is_err());
    }

    #[test]
    fn malformed_sdist_empty_name_errors() {
        assert!(classify_filename("-1.0.tar.gz").is_err());
    }

    #[test]
    fn malformed_sdist_empty_version_errors() {
        assert!(classify_filename("name-.tar.gz").is_err());
    }

    #[test]
    fn malformed_wheel_filename_errors() {
        // Wheel filename layout requires 5 dash-separated parts.
        assert!(classify_filename("badwheel.whl").is_err());
    }

    #[test]
    fn classify_many_skips_unknown_extensions() {
        let names = [
            "requests-2.31.0-py3-none-any.whl",
            "README.md",
            "setuptools-68.2.0.zip",
            "Pipfile",
        ];
        let entries = classify_many(names).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "requests");
        assert_eq!(entries[1].name, "setuptools");
    }

    #[test]
    fn classify_many_surfaces_first_hard_error() {
        // README is skipped; malformed sdist errors.
        let names = ["README.md", "noversion.tar.gz", "ok-1.0.tar.gz"];
        assert!(classify_many(names).is_err());
    }

    #[test]
    fn group_by_name_groups_multiple_versions_of_same_package() {
        let entries = classify_many([
            "requests-2.30.0-py3-none-any.whl",
            "requests-2.31.0-py3-none-any.whl",
            "requests-2.31.0.tar.gz",
            "click-8.1.7-py3-none-any.whl",
        ])
        .unwrap();
        let grouped = group_by_name(entries);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("requests").unwrap().len(), 3);
        assert_eq!(grouped.get("click").unwrap().len(), 1);
    }

    #[test]
    fn group_by_name_preserves_input_order_within_bucket() {
        let entries = classify_many(["x-1.0.tar.gz", "x-2.0.tar.gz", "x-3.0.tar.gz"]).unwrap();
        let grouped = group_by_name(entries);
        let x = grouped.get("x").unwrap();
        assert_eq!(x[0].version, "1.0");
        assert_eq!(x[1].version, "2.0");
        assert_eq!(x[2].version, "3.0");
    }

    #[test]
    fn group_by_name_buckets_are_alphabetical() {
        let entries =
            classify_many(["zope-1.0.tar.gz", "alpha-1.0.tar.gz", "middle-1.0.tar.gz"]).unwrap();
        let grouped = group_by_name(entries);
        let keys: Vec<&str> = grouped.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["alpha", "middle", "zope"]);
    }

    #[test]
    fn binary_wheel_filename_with_build_tag_classifies() {
        let e = classify_filename("numpy-1.26.0-1-cp311-cp311-manylinux_2_17_x86_64.whl")
            .unwrap()
            .unwrap();
        assert_eq!(e.name, "numpy");
        assert_eq!(e.version, "1.26.0");
        assert_eq!(e.kind, ArtifactKind::Wheel);
    }

    #[test]
    fn artifact_kind_as_str_round_trip() {
        assert_eq!(ArtifactKind::Wheel.as_str(), "wheel");
        assert_eq!(ArtifactKind::Sdist.as_str(), "sdist");
    }

    #[test]
    fn realistic_find_links_directory_contents() {
        // Mirror what a real wheelhouse looks like.
        let entries = classify_many([
            "anyio-4.0.0-py3-none-any.whl",
            "anyio-4.0.0.tar.gz",
            "sniffio-1.3.0-py3-none-any.whl",
            "sniffio-1.3.0.tar.gz",
            "idna-3.4-py3-none-any.whl",
            "idna-3.4.tar.gz",
            "h11-0.14.0-py3-none-any.whl",
            "h11-0.14.0.tar.gz",
            "httpcore-1.0.2-py3-none-any.whl",
            "httpcore-1.0.2.tar.gz",
            "httpx-0.25.2-py3-none-any.whl",
            "httpx-0.25.2.tar.gz",
            "README.txt",
            "checksums.txt",
        ])
        .unwrap();
        assert_eq!(entries.len(), 12);
        let grouped = group_by_name(entries);
        assert_eq!(grouped.len(), 6);
        for name in &["anyio", "sniffio", "idna", "h11", "httpcore", "httpx"] {
            let v = grouped.get(*name).unwrap();
            assert_eq!(v.len(), 2);
            // Each package has one wheel + one sdist.
            let kinds: std::collections::HashSet<ArtifactKind> = v.iter().map(|e| e.kind).collect();
            assert_eq!(kinds.len(), 2);
        }
    }

    #[test]
    fn sdist_with_underscore_in_name_is_pep503_normalized() {
        let e = classify_filename("backports_zoneinfo-0.2.1.tar.gz")
            .unwrap()
            .unwrap();
        assert_eq!(e.raw_name, "backports_zoneinfo");
        assert_eq!(e.name, "backports-zoneinfo");
    }
}
