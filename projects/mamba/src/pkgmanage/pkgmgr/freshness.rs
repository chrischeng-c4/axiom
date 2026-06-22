// Yanked-release filtering + cache freshness (Tick 28).
//
// Two small primitives the resolver/installer compose later. Both are
// pure data layer — no HTTP, no global state — so they can be unit-tested
// without spawning anything.
//
// Yanked policy mirrors uv:
//   * PEP 592 marks an uploaded release as "yanked" with an optional
//     human reason. Yanked releases stay reachable on the index but
//     resolvers must not pick them implicitly. A user *may* still install
//     a yanked version if they pin to it exactly — the policy default is
//     "AllowPinnedOnly": invisible to fresh resolution, visible to a
//     lockfile or `==X.Y.Z` user pin.
//   * `Forbid` rejects yanked even when pinned — useful for security
//     re-resolves where the user wants to be loud about it.
//   * `Allow` is `--allow-yanked-everywhere`: treat yanked as normal.
//     Almost no one wants this; we ship it for parity.
//
// Freshness:
//   * Metadata cache entries grow stale. uv's default TTL for the JSON
//     index is 10 minutes (`UV_CACHE_TTL`-ish, configurable). We return
//     a verdict rather than a bool so callers can log "age N seconds /
//     TTL M seconds" diagnostically and decide their own refetch.
//   * `Missing` is distinct from `Stale`: missing means "nothing on disk
//     to compare to". Useful so the caller can pick a different log line
//     ("first fetch" vs "refresh").

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::pkgmanage::pkgmgr::types::{IndexError, ReleaseFile};

// --- yanked filtering -------------------------------------------------------

/// What to do with releases marked `yanked = true` on the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankedPolicy {
    /// Default: hide yanked from resolution, *unless* the user has
    /// pinned to that exact version (lockfile entry or `==X.Y.Z`).
    AllowPinnedOnly,
    /// Strict: reject yanked unconditionally. Pin or not, it's out.
    Forbid,
    /// Permissive: treat yanked exactly like a normal release.
    /// Equivalent to uv's `--allow-yanked-everywhere`.
    Allow,
}

impl Default for YankedPolicy {
    fn default() -> Self {
        Self::AllowPinnedOnly
    }
}

/// One yanked release that was filtered out. Captured for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YankedFiltering {
    /// Version string parsed from `ReleaseFile::filename` by the caller.
    pub version: String,
    /// Filename of the artifact that was filtered.
    pub filename: String,
    /// PEP 592 reason text, if the index supplied one.
    pub reason: Option<String>,
    /// True when the user had pinned to this exact version. Combined
    /// with `Forbid` policy this becomes an error from the caller.
    pub pinned: bool,
}

/// Result of filtering a slice of release files.
#[derive(Debug, Clone, Default)]
pub struct YankedDecision {
    /// Releases that survived the policy gate.
    pub usable: Vec<ReleaseFile>,
    /// Yanked releases that were excluded (for logging/UX).
    pub filtered: Vec<YankedFiltering>,
    /// Human-readable advisories. Currently used to flag "you pinned
    /// to a yanked version" so the caller can surface it once at the
    /// resolver boundary.
    pub warnings: Vec<String>,
}

/// Apply a yanked policy to a release set.
///
/// `version_of` extracts the version string from each `ReleaseFile`. We
/// take it as a closure because two callers can disagree on canonical
/// form (filename-derived vs index-provided); this module stays agnostic.
///
/// `pinned_versions` is the set of exact versions the user explicitly
/// asked for (lockfile or `==`). When the policy is `AllowPinnedOnly` a
/// yanked release survives iff its version sits in this set.
pub fn filter_yanked<'a, F>(
    files: &'a [ReleaseFile],
    policy: YankedPolicy,
    pinned_versions: &BTreeSet<String>,
    version_of: F,
) -> YankedDecision
where
    F: Fn(&'a ReleaseFile) -> &'a str,
{
    let mut out = YankedDecision::default();
    for file in files {
        if !file.yanked {
            out.usable.push(file.clone());
            continue;
        }
        let version = version_of(file).to_string();
        let pinned = pinned_versions.contains(&version);
        let keep = match policy {
            YankedPolicy::Allow => true,
            YankedPolicy::AllowPinnedOnly => pinned,
            YankedPolicy::Forbid => false,
        };
        if keep {
            out.usable.push(file.clone());
            if pinned {
                let reason = file
                    .yanked_reason
                    .clone()
                    .unwrap_or_else(|| "no reason given".into());
                out.warnings.push(format!(
                    "using yanked release {version} (pinned); reason: {reason}"
                ));
            }
        } else {
            out.filtered.push(YankedFiltering {
                version,
                filename: file.filename.clone(),
                reason: file.yanked_reason.clone(),
                pinned,
            });
        }
    }
    out
}

/// Convenience: return `Err(IndexError::YankedRelease)` if `usable` is
/// empty and every filtered entry shared the same `name`. The caller
/// generally knows the package name; we accept it as a parameter rather
/// than try to derive it from filenames.
pub fn enforce_at_least_one_usable(
    name: &str,
    decision: &YankedDecision,
) -> Result<(), IndexError> {
    if !decision.usable.is_empty() {
        return Ok(());
    }
    // Pick the highest-versioned filtered entry for the error message.
    // Lexicographic max is fine here: error text only.
    if let Some(worst) = decision
        .filtered
        .iter()
        .max_by(|a, b| a.version.cmp(&b.version))
    {
        return Err(IndexError::YankedRelease {
            name: name.to_string(),
            version: worst.version.clone(),
        });
    }
    Err(IndexError::NotFound {
        name: name.to_string(),
    })
}

// --- freshness --------------------------------------------------------------

/// Caller-supplied policy: how long is a cached metadata file allowed to
/// live before we consider it stale?
#[derive(Debug, Clone, Copy)]
pub struct FreshnessPolicy {
    /// Time-to-live for the cache entry.
    pub ttl: Duration,
    /// "Now" — injected for deterministic tests. Production callers pass
    /// `SystemTime::now()`.
    pub now: SystemTime,
}

impl FreshnessPolicy {
    /// 10-minute default, matching uv's metadata cache.
    pub fn default_ttl() -> Duration {
        Duration::from_secs(600)
    }
}

/// Verdict from a freshness probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FreshnessVerdict {
    /// File exists and is younger than the TTL.
    Fresh { age: Duration },
    /// File exists but is older than the TTL.
    Stale { age: Duration, ttl: Duration },
    /// File does not exist on disk.
    Missing,
}

impl FreshnessVerdict {
    /// True iff the caller should *use* the cached file as-is (no
    /// refetch needed).
    pub fn is_fresh(&self) -> bool {
        matches!(self, FreshnessVerdict::Fresh { .. })
    }
    /// True iff the caller should refetch (either Missing or Stale).
    pub fn requires_refetch(&self) -> bool {
        !self.is_fresh()
    }
}

/// Inspect a metadata cache file and classify its freshness.
///
/// `path` may be absent on disk — that's reported as `Missing`, not as
/// an error. A real I/O error (permission denied, broken FS) is still
/// surfaced as `IndexError::CacheIo`.
pub fn check_metadata_freshness(
    path: &Path,
    policy: &FreshnessPolicy,
) -> Result<FreshnessVerdict, IndexError> {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FreshnessVerdict::Missing);
        }
        Err(err) => {
            return Err(IndexError::CacheIo {
                path: path.display().to_string(),
                detail: format!("stat: {err}"),
            });
        }
    };
    let modified = meta.modified().map_err(|err| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: format!("reading mtime: {err}"),
    })?;
    let age = policy
        .now
        .duration_since(modified)
        .unwrap_or(Duration::ZERO);
    if age <= policy.ttl {
        Ok(FreshnessVerdict::Fresh { age })
    } else {
        Ok(FreshnessVerdict::Stale {
            age,
            ttl: policy.ttl,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;
    use tempfile::TempDir;

    fn mk_file(filename: &str, yanked: bool, reason: Option<&str>) -> ReleaseFile {
        ReleaseFile {
            filename: filename.into(),
            url: format!("https://example.test/{filename}"),
            hash: Default::default(),
            requires_python: None,
            size: None,
            upload_time: None,
            yanked,
            yanked_reason: reason.map(|s| s.to_string()),
            dist_info_metadata: serde_json::Value::Null,
            source: None,
        }
    }

    // Derive version from "name-X.Y.Z-…" filename for tests.
    fn version_from_filename(f: &ReleaseFile) -> &str {
        // foo-1.2.3-py3-none-any.whl  -> 1.2.3
        let stem = f.filename.split('-').nth(1).unwrap_or("");
        stem
    }

    #[test]
    fn filter_keeps_non_yanked_unchanged() {
        let files = vec![
            mk_file("foo-1.0.0-py3-none-any.whl", false, None),
            mk_file("foo-2.0.0-py3-none-any.whl", false, None),
        ];
        let pinned = BTreeSet::new();
        let d = filter_yanked(
            &files,
            YankedPolicy::AllowPinnedOnly,
            &pinned,
            version_from_filename,
        );
        assert_eq!(d.usable.len(), 2);
        assert!(d.filtered.is_empty());
        assert!(d.warnings.is_empty());
    }

    #[test]
    fn allow_pinned_only_hides_yanked_unless_pinned() {
        let files = vec![
            mk_file("foo-1.0.0-py3-none-any.whl", false, None),
            mk_file("foo-1.5.0-py3-none-any.whl", true, Some("broken metadata")),
            mk_file("foo-2.0.0-py3-none-any.whl", false, None),
        ];
        let pinned = BTreeSet::new();
        let d = filter_yanked(
            &files,
            YankedPolicy::AllowPinnedOnly,
            &pinned,
            version_from_filename,
        );
        assert_eq!(d.usable.len(), 2);
        assert_eq!(d.filtered.len(), 1);
        assert_eq!(d.filtered[0].version, "1.5.0");
        assert_eq!(d.filtered[0].reason.as_deref(), Some("broken metadata"));
        assert!(!d.filtered[0].pinned);
    }

    #[test]
    fn allow_pinned_only_keeps_yanked_when_user_pinned() {
        let files = vec![mk_file(
            "foo-1.5.0-py3-none-any.whl",
            true,
            Some("CVE-2024-x"),
        )];
        let mut pinned = BTreeSet::new();
        pinned.insert("1.5.0".to_string());
        let d = filter_yanked(
            &files,
            YankedPolicy::AllowPinnedOnly,
            &pinned,
            version_from_filename,
        );
        assert_eq!(d.usable.len(), 1);
        assert!(d.filtered.is_empty());
        assert_eq!(d.warnings.len(), 1, "should emit a 'using yanked' warning");
        assert!(d.warnings[0].contains("1.5.0"));
        assert!(d.warnings[0].contains("CVE-2024-x"));
    }

    #[test]
    fn forbid_rejects_yanked_even_when_pinned() {
        let files = vec![mk_file("foo-1.5.0-py3-none-any.whl", true, None)];
        let mut pinned = BTreeSet::new();
        pinned.insert("1.5.0".to_string());
        let d = filter_yanked(&files, YankedPolicy::Forbid, &pinned, version_from_filename);
        assert!(d.usable.is_empty());
        assert_eq!(d.filtered.len(), 1);
        assert!(d.filtered[0].pinned);
    }

    #[test]
    fn allow_treats_yanked_as_normal() {
        let files = vec![
            mk_file("foo-1.0.0-py3-none-any.whl", false, None),
            mk_file("foo-1.5.0-py3-none-any.whl", true, Some("oops")),
        ];
        let pinned = BTreeSet::new();
        let d = filter_yanked(&files, YankedPolicy::Allow, &pinned, version_from_filename);
        assert_eq!(d.usable.len(), 2);
        assert!(d.filtered.is_empty());
        assert!(d.warnings.is_empty(), "Allow mode should not warn");
    }

    #[test]
    fn no_yanked_reason_renders_default_text() {
        let files = vec![mk_file("foo-1.5.0-py3-none-any.whl", true, None)];
        let mut pinned = BTreeSet::new();
        pinned.insert("1.5.0".to_string());
        let d = filter_yanked(
            &files,
            YankedPolicy::AllowPinnedOnly,
            &pinned,
            version_from_filename,
        );
        assert_eq!(d.warnings.len(), 1);
        assert!(d.warnings[0].contains("no reason given"));
    }

    #[test]
    fn enforce_at_least_one_usable_passes_when_usable_nonempty() {
        let mut decision = YankedDecision::default();
        decision
            .usable
            .push(mk_file("foo-1.0.0-py3-none-any.whl", false, None));
        enforce_at_least_one_usable("foo", &decision).unwrap();
    }

    #[test]
    fn enforce_at_least_one_usable_errors_with_yanked_diagnostic() {
        let mut decision = YankedDecision::default();
        decision.filtered.push(YankedFiltering {
            version: "1.5.0".into(),
            filename: "foo-1.5.0-py3-none-any.whl".into(),
            reason: Some("broken".into()),
            pinned: false,
        });
        let err = enforce_at_least_one_usable("foo", &decision).unwrap_err();
        match err {
            IndexError::YankedRelease { name, version } => {
                assert_eq!(name, "foo");
                assert_eq!(version, "1.5.0");
            }
            other => panic!("expected YankedRelease, got {other:?}"),
        }
    }

    #[test]
    fn enforce_at_least_one_usable_errors_not_found_when_nothing_at_all() {
        let decision = YankedDecision::default();
        let err = enforce_at_least_one_usable("foo", &decision).unwrap_err();
        match err {
            IndexError::NotFound { name } => assert_eq!(name, "foo"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    // --- freshness tests ---

    #[test]
    fn freshness_missing_when_file_absent() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("never-written.json");
        let policy = FreshnessPolicy {
            ttl: Duration::from_secs(60),
            now: SystemTime::now(),
        };
        let v = check_metadata_freshness(&missing, &policy).unwrap();
        assert_eq!(v, FreshnessVerdict::Missing);
        assert!(v.requires_refetch());
        assert!(!v.is_fresh());
    }

    #[test]
    fn freshness_fresh_when_mtime_within_ttl() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("meta.json");
        fs::write(&p, b"{}").unwrap();
        let mtime = fs::metadata(&p).unwrap().modified().unwrap();
        let policy = FreshnessPolicy {
            ttl: Duration::from_secs(600),
            now: mtime + Duration::from_secs(30),
        };
        let v = check_metadata_freshness(&p, &policy).unwrap();
        match v {
            FreshnessVerdict::Fresh { age } => {
                assert!(age <= Duration::from_secs(31));
            }
            other => panic!("expected Fresh, got {other:?}"),
        }
    }

    #[test]
    fn freshness_stale_when_mtime_past_ttl() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("meta.json");
        fs::write(&p, b"{}").unwrap();
        let mtime = fs::metadata(&p).unwrap().modified().unwrap();
        let policy = FreshnessPolicy {
            ttl: Duration::from_secs(60),
            now: mtime + Duration::from_secs(900),
        };
        let v = check_metadata_freshness(&p, &policy).unwrap();
        match v {
            FreshnessVerdict::Stale { age, ttl } => {
                assert!(age >= Duration::from_secs(800));
                assert_eq!(ttl, Duration::from_secs(60));
            }
            other => panic!("expected Stale, got {other:?}"),
        }
        assert!(v.requires_refetch());
    }

    #[test]
    fn freshness_fresh_when_now_before_mtime() {
        // Clock skew: caller's `now` is somehow earlier than the cached
        // file's mtime. Treat as fresh (age = 0); never panic.
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("meta.json");
        fs::write(&p, b"{}").unwrap();
        let mtime = fs::metadata(&p).unwrap().modified().unwrap();
        let policy = FreshnessPolicy {
            ttl: Duration::from_secs(60),
            now: mtime - Duration::from_secs(30),
        };
        let v = check_metadata_freshness(&p, &policy).unwrap();
        assert!(
            v.is_fresh(),
            "expected Fresh on backward clock skew, got {v:?}"
        );
    }

    #[test]
    fn freshness_policy_default_ttl_is_ten_minutes() {
        assert_eq!(FreshnessPolicy::default_ttl(), Duration::from_secs(600));
    }
}
