// Cache prune + size policy (Tick 38).
//
// Mirrors `uv cache clean` / `uv cache prune` / `uv cache info`. The
// existing `cache` module owns the on-disk *layout* (paths, TTL
// constants, content-addressed sharding); this module owns the
// *housekeeping* layer that runs on top:
//
//   * enumerate entries — walk `metadata/`, `artifacts/`, `content/`
//     and produce a `CacheInventory` of (path, size_bytes, mtime,
//     category) tuples.
//   * apply policies — `PrunePolicy` captures the user's intent
//     (max age, max total size, "everything matching a name", or just
//     a dry-run inspection) and `plan_prune` turns that into a
//     deterministic list of `PrunePlanEntry { path, reason }`.
//   * execute — `apply_prune_plan` walks the plan, removes files (or
//     reports what would be removed for `--dry-run`), and returns a
//     `PruneSummary` with totals.
//
// The split lets us unit-test the policy without touching real disk,
// while the driver layer is exercised end-to-end through a tempdir.
// All decisions are deterministic given the inventory snapshot —
// concurrent writers are out of scope (matching uv's own model).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::pkgmanage::pkgmgr::cache::normalize_name;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Coarse classification of cache entries so policies can target one
/// pile without disturbing another (e.g. "prune metadata older than
/// 1h but never touch downloaded wheels").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheCategory {
    /// `metadata/<name>/{json|simple}-api.json`
    Metadata,
    /// `artifacts/<name>/<filename>`
    Artifact,
    /// `content/<2>/<sha256>`
    ContentAddressed,
    /// Anything else under the cache root we still want to enumerate
    /// for `cache info` totals, but won't touch unless the user opts
    /// in with `PrunePolicy::all_unknown_too`.
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub mtime: SystemTime,
    pub category: CacheCategory,
    /// PEP 503-normalized package name when extractable from the
    /// path (Metadata + Artifact only). Used by name-targeted prunes.
    pub package: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheInventory {
    pub root: PathBuf,
    pub entries: Vec<CacheEntry>,
}

impl CacheInventory {
    pub fn total_bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.size_bytes).sum()
    }
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    pub fn count_in(&self, cat: CacheCategory) -> usize {
        self.entries.iter().filter(|e| e.category == cat).count()
    }
    pub fn bytes_in(&self, cat: CacheCategory) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.category == cat)
            .map(|e| e.size_bytes)
            .sum()
    }
}

/// Walk a cache directory and produce an inventory snapshot. Returns
/// an empty inventory if `cache_dir` doesn't exist — that's the same
/// thing as "nothing cached", not an error.
pub fn enumerate_cache(cache_dir: &Path) -> Result<CacheInventory, IndexError> {
    let mut entries = Vec::new();
    if !cache_dir.exists() {
        return Ok(CacheInventory {
            root: cache_dir.to_path_buf(),
            entries,
        });
    }
    walk_dir(cache_dir, cache_dir, &mut entries)?;
    // Sort by path so the inventory is deterministic for tests and
    // for "cache info" output.
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(CacheInventory {
        root: cache_dir.to_path_buf(),
        entries,
    })
}

fn walk_dir(root: &Path, dir: &Path, out: &mut Vec<CacheEntry>) -> Result<(), IndexError> {
    let read = fs::read_dir(dir).map_err(|e| IndexError::ParseError {
        url: dir.display().to_string(),
        detail: format!("reading cache dir: {e}"),
    })?;
    for entry in read {
        let entry = entry.map_err(|e| IndexError::ParseError {
            url: dir.display().to_string(),
            detail: format!("listing cache dir: {e}"),
        })?;
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // race with concurrent prune — skip
        };
        if meta.is_dir() {
            walk_dir(root, &path, out)?;
            continue;
        }
        if !meta.is_file() {
            continue;
        }
        let size_bytes = meta.len();
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let (category, package) = classify(root, &path);
        out.push(CacheEntry {
            path,
            size_bytes,
            mtime,
            category,
            package,
        });
    }
    Ok(())
}

fn classify(root: &Path, path: &Path) -> (CacheCategory, Option<String>) {
    let Ok(rel) = path.strip_prefix(root) else {
        return (CacheCategory::Other, None);
    };
    let mut comps = rel.components();
    match comps.next().and_then(|c| c.as_os_str().to_str()) {
        Some("metadata") => {
            let name = comps
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .map(|s| s.to_string());
            (CacheCategory::Metadata, name)
        }
        Some("artifacts") => {
            let name = comps
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .map(|s| s.to_string());
            (CacheCategory::Artifact, name)
        }
        Some("content") => (CacheCategory::ContentAddressed, None),
        _ => (CacheCategory::Other, None),
    }
}

/// Strategy for choosing which entries to remove. Multiple constraints
/// stack: each one filters the candidate set further.
#[derive(Debug, Clone, Default)]
pub struct PrunePolicy {
    /// Drop entries older than this. None = no age limit.
    pub max_age: Option<Duration>,
    /// Cap on total cache size in bytes. When the inventory is over
    /// budget, we evict oldest-first until the total fits. None = no
    /// size cap.
    pub max_total_bytes: Option<u64>,
    /// If non-empty, only entries whose `package` matches one of these
    /// PEP 503-normalized names are eligible. Empty = consider all.
    pub only_packages: Vec<String>,
    /// If non-empty, only entries in these categories are eligible.
    /// Empty = all categories except Other (Other requires
    /// `all_unknown_too`).
    pub categories: Vec<CacheCategory>,
    /// Opt-in for cleaning the `Other` bucket. Off by default so we
    /// never delete user-owned files that happen to live under the
    /// cache root.
    pub all_unknown_too: bool,
    /// When true, the "remove everything that matches" wipe semantics
    /// of `uv cache clean` — every eligible entry is selected.
    pub wipe: bool,
}

impl PrunePolicy {
    /// Convenience: "delete the cache entirely except for the `Other`
    /// bucket".
    pub fn clean_all() -> Self {
        PrunePolicy {
            wipe: true,
            ..Default::default()
        }
    }
    /// Convenience: prune entries older than `age`.
    pub fn older_than(age: Duration) -> Self {
        PrunePolicy {
            max_age: Some(age),
            ..Default::default()
        }
    }
}

/// One scheduled removal. `reason` is a short tag for surfacing
/// through `cache info --json` / debug logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunePlanEntry {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub reason: PruneReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PruneReason {
    Wipe,
    OlderThan,
    OverSizeBudget,
    NameMatch,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrunePlan {
    pub entries: Vec<PrunePlanEntry>,
}

impl PrunePlan {
    pub fn total_bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.size_bytes).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Pure decision layer: given an inventory + policy + "now",
/// produce the deterministic list of files to remove. No I/O.
pub fn plan_prune(inventory: &CacheInventory, policy: &PrunePolicy, now: SystemTime) -> PrunePlan {
    let allowed_cats: HashSet<CacheCategory> = if policy.categories.is_empty() {
        if policy.all_unknown_too {
            [
                CacheCategory::Metadata,
                CacheCategory::Artifact,
                CacheCategory::ContentAddressed,
                CacheCategory::Other,
            ]
            .into_iter()
            .collect()
        } else {
            [
                CacheCategory::Metadata,
                CacheCategory::Artifact,
                CacheCategory::ContentAddressed,
            ]
            .into_iter()
            .collect()
        }
    } else {
        policy.categories.iter().copied().collect()
    };

    let name_filter: HashSet<String> = policy
        .only_packages
        .iter()
        .map(|n| normalize_name(n))
        .collect();

    let mut chosen: Vec<&CacheEntry> = inventory
        .entries
        .iter()
        .filter(|e| allowed_cats.contains(&e.category))
        .filter(|e| {
            if name_filter.is_empty() {
                return true;
            }
            e.package
                .as_deref()
                .map(|n| name_filter.contains(&normalize_name(n)))
                .unwrap_or(false)
        })
        .collect();

    if policy.wipe || !name_filter.is_empty() {
        let reason = if policy.wipe {
            PruneReason::Wipe
        } else {
            PruneReason::NameMatch
        };
        return PrunePlan {
            entries: chosen
                .into_iter()
                .map(|e| PrunePlanEntry {
                    path: e.path.clone(),
                    size_bytes: e.size_bytes,
                    reason,
                })
                .collect(),
        };
    }

    let mut planned: Vec<PrunePlanEntry> = Vec::new();
    let mut planned_paths: HashSet<PathBuf> = HashSet::new();

    if let Some(age) = policy.max_age {
        for e in &chosen {
            let elapsed = now.duration_since(e.mtime).unwrap_or(Duration::ZERO);
            if elapsed > age {
                if planned_paths.insert(e.path.clone()) {
                    planned.push(PrunePlanEntry {
                        path: e.path.clone(),
                        size_bytes: e.size_bytes,
                        reason: PruneReason::OlderThan,
                    });
                }
            }
        }
    }

    if let Some(budget) = policy.max_total_bytes {
        // Evict oldest first until total fits the budget. We only
        // consider entries not already scheduled (they're going away
        // anyway).
        let already: HashSet<PathBuf> = planned_paths.clone();
        chosen.sort_by_key(|e| e.mtime);
        let surviving_total: u64 = chosen
            .iter()
            .filter(|e| !already.contains(&e.path))
            .map(|e| e.size_bytes)
            .sum();
        if surviving_total > budget {
            let mut over = surviving_total - budget;
            for e in &chosen {
                if over == 0 {
                    break;
                }
                if already.contains(&e.path) {
                    continue;
                }
                if planned_paths.insert(e.path.clone()) {
                    planned.push(PrunePlanEntry {
                        path: e.path.clone(),
                        size_bytes: e.size_bytes,
                        reason: PruneReason::OverSizeBudget,
                    });
                    over = over.saturating_sub(e.size_bytes);
                }
            }
        }
    }

    // Deterministic output ordering.
    planned.sort_by(|a, b| a.path.cmp(&b.path));
    PrunePlan { entries: planned }
}

/// Outcome of running a prune plan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PruneSummary {
    pub removed: usize,
    pub bytes_freed: u64,
    pub failures: Vec<(PathBuf, String)>,
    pub dry_run: bool,
}

/// Apply a prune plan. When `dry_run` is true, no files are touched
/// and `removed` reflects what *would* be removed.
pub fn apply_prune_plan(plan: &PrunePlan, dry_run: bool) -> PruneSummary {
    let mut summary = PruneSummary {
        dry_run,
        ..Default::default()
    };
    for e in &plan.entries {
        if dry_run {
            summary.removed += 1;
            summary.bytes_freed += e.size_bytes;
            continue;
        }
        match fs::remove_file(&e.path) {
            Ok(()) => {
                summary.removed += 1;
                summary.bytes_freed += e.size_bytes;
            }
            Err(err) => {
                summary.failures.push((e.path.clone(), err.to_string()));
            }
        }
    }
    summary
}

/// After a prune, walk the cache and drop directories that became
/// empty. Returns the number of directories removed. Errors are
/// silently swallowed — a stale empty dir is not a fatal condition.
pub fn collapse_empty_dirs(cache_dir: &Path) -> usize {
    fn walk(dir: &Path, removed: &mut usize) {
        let Ok(read) = fs::read_dir(dir) else { return };
        let entries: Vec<_> = read.flatten().collect();
        for e in &entries {
            let path = e.path();
            if path.is_dir() {
                walk(&path, removed);
            }
        }
        // Re-read after recursing: children may have just gone away.
        if let Ok(mut iter) = fs::read_dir(dir) {
            if iter.next().is_none() {
                if fs::remove_dir(dir).is_ok() {
                    *removed += 1;
                }
            }
        }
    }
    if !cache_dir.exists() {
        return 0;
    }
    let mut removed = 0;
    // Walk top-level subdirs but never delete the cache root itself.
    let Ok(read) = fs::read_dir(cache_dir) else {
        return 0;
    };
    for e in read.flatten() {
        let path = e.path();
        if path.is_dir() {
            walk(&path, &mut removed);
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn touch(path: &Path, size: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, vec![0u8; size]).unwrap();
    }

    #[test]
    fn enumerate_missing_dir_returns_empty_not_error() {
        let inv = enumerate_cache(Path::new("/nonexistent/path/here")).unwrap();
        assert!(inv.entries.is_empty());
    }

    #[test]
    fn enumerate_classifies_metadata_artifact_content() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(&root.join("metadata/requests/json-api.json"), 100);
        touch(&root.join("artifacts/requests/req-2.31.0.whl"), 500);
        touch(
            &root.join(
                "content/ab/abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            ),
            900,
        );
        touch(&root.join("rogue/note.txt"), 50);
        let inv = enumerate_cache(root).unwrap();
        assert_eq!(inv.count(), 4);
        assert_eq!(inv.count_in(CacheCategory::Metadata), 1);
        assert_eq!(inv.count_in(CacheCategory::Artifact), 1);
        assert_eq!(inv.count_in(CacheCategory::ContentAddressed), 1);
        assert_eq!(inv.count_in(CacheCategory::Other), 1);
        assert_eq!(inv.total_bytes(), 1550);
        let meta = inv
            .entries
            .iter()
            .find(|e| e.category == CacheCategory::Metadata)
            .unwrap();
        assert_eq!(meta.package.as_deref(), Some("requests"));
    }

    fn inventory(now: SystemTime, files: &[(CacheCategory, &str, u64, u64)]) -> CacheInventory {
        // Synthetic inventory bypassing the FS — for pure policy tests.
        let entries = files
            .iter()
            .map(|(cat, path, size, secs_old)| CacheEntry {
                path: PathBuf::from(path),
                size_bytes: *size,
                mtime: now - Duration::from_secs(*secs_old),
                category: *cat,
                package: match cat {
                    CacheCategory::Metadata | CacheCategory::Artifact => {
                        // Pull "package" segment from the synthetic path.
                        PathBuf::from(path)
                            .components()
                            .nth(1)
                            .and_then(|c| c.as_os_str().to_str())
                            .map(|s| s.to_string())
                    }
                    _ => None,
                },
            })
            .collect();
        CacheInventory {
            root: PathBuf::from("/cache"),
            entries,
        }
    }

    #[test]
    fn plan_prune_max_age_picks_old_entries_only() {
        let now = SystemTime::now();
        let inv = inventory(
            now,
            &[
                (
                    CacheCategory::Metadata,
                    "metadata/a/json-api.json",
                    100,
                    7200,
                ),
                (CacheCategory::Metadata, "metadata/b/json-api.json", 100, 60),
            ],
        );
        let plan = plan_prune(
            &inv,
            &PrunePolicy::older_than(Duration::from_secs(3600)),
            now,
        );
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan.entries[0].path,
            PathBuf::from("metadata/a/json-api.json")
        );
        assert_eq!(plan.entries[0].reason, PruneReason::OlderThan);
    }

    #[test]
    fn plan_prune_max_total_bytes_evicts_oldest_first() {
        let now = SystemTime::now();
        let inv = inventory(
            now,
            &[
                (CacheCategory::Artifact, "artifacts/x/a.whl", 1000, 30),
                (CacheCategory::Artifact, "artifacts/x/b.whl", 1000, 60),
                (CacheCategory::Artifact, "artifacts/x/c.whl", 1000, 90),
            ],
        );
        let plan = plan_prune(
            &inv,
            &PrunePolicy {
                max_total_bytes: Some(1500),
                ..Default::default()
            },
            now,
        );
        // We need to evict 1500 bytes (3000 -> 1500). Oldest = 90s
        // first, then 60s. After dropping both, total is 1000 ≤ 1500.
        assert_eq!(plan.len(), 2);
        let paths: HashSet<PathBuf> = plan.entries.iter().map(|e| e.path.clone()).collect();
        assert!(paths.contains(&PathBuf::from("artifacts/x/b.whl")));
        assert!(paths.contains(&PathBuf::from("artifacts/x/c.whl")));
        for e in &plan.entries {
            assert_eq!(e.reason, PruneReason::OverSizeBudget);
        }
    }

    #[test]
    fn plan_prune_wipe_takes_every_eligible_entry() {
        let now = SystemTime::now();
        let inv = inventory(
            now,
            &[
                (CacheCategory::Metadata, "metadata/a/json-api.json", 100, 1),
                (CacheCategory::Artifact, "artifacts/a/x.whl", 200, 1),
                (CacheCategory::Other, "rogue/note.txt", 99, 1),
            ],
        );
        let plan = plan_prune(&inv, &PrunePolicy::clean_all(), now);
        // Default wipe spares the Other bucket.
        assert_eq!(plan.len(), 2);
        for e in &plan.entries {
            assert_eq!(e.reason, PruneReason::Wipe);
        }
    }

    #[test]
    fn plan_prune_wipe_with_all_unknown_too_takes_other_bucket() {
        let now = SystemTime::now();
        let inv = inventory(
            now,
            &[
                (CacheCategory::Metadata, "metadata/a/json-api.json", 100, 1),
                (CacheCategory::Other, "rogue/note.txt", 99, 1),
            ],
        );
        let plan = plan_prune(
            &inv,
            &PrunePolicy {
                wipe: true,
                all_unknown_too: true,
                ..Default::default()
            },
            now,
        );
        assert_eq!(plan.len(), 2);
    }

    #[test]
    fn plan_prune_name_filter_targets_matching_packages_only() {
        let now = SystemTime::now();
        let inv = inventory(
            now,
            &[
                (
                    CacheCategory::Metadata,
                    "metadata/requests/json-api.json",
                    10,
                    1,
                ),
                (
                    CacheCategory::Metadata,
                    "metadata/flask/json-api.json",
                    10,
                    1,
                ),
                (CacheCategory::Artifact, "artifacts/requests/req.whl", 50, 1),
            ],
        );
        let plan = plan_prune(
            &inv,
            &PrunePolicy {
                only_packages: vec!["Requests".to_string()],
                ..Default::default()
            },
            now,
        );
        // No max_age + no max_total_bytes + no wipe → name_match
        // path on its own selects every entry tagged with that name.
        assert_eq!(plan.len(), 2);
        for e in &plan.entries {
            assert_eq!(e.reason, PruneReason::NameMatch);
        }
    }

    #[test]
    fn plan_prune_combines_age_and_size_without_double_listing() {
        let now = SystemTime::now();
        let inv = inventory(
            now,
            &[
                // Old AND big — eligible under both clauses, must appear once.
                (CacheCategory::Artifact, "artifacts/x/a.whl", 5000, 7200),
                (CacheCategory::Artifact, "artifacts/x/b.whl", 100, 60),
            ],
        );
        let plan = plan_prune(
            &inv,
            &PrunePolicy {
                max_age: Some(Duration::from_secs(3600)),
                max_total_bytes: Some(50), // forces extra eviction
                ..Default::default()
            },
            now,
        );
        // a.whl should appear once (OlderThan wins because it's checked first).
        let paths: Vec<&PathBuf> = plan.entries.iter().map(|e| &e.path).collect();
        let count_a = paths
            .iter()
            .filter(|p| ***p == PathBuf::from("artifacts/x/a.whl"))
            .count();
        assert_eq!(count_a, 1);
        // After dropping a (5000 B), surviving total = 100 B > 50 B,
        // so b.whl is also scheduled with OverSizeBudget.
        assert_eq!(plan.len(), 2);
        let reasons: HashSet<PruneReason> = plan.entries.iter().map(|e| e.reason).collect();
        assert!(reasons.contains(&PruneReason::OlderThan));
        assert!(reasons.contains(&PruneReason::OverSizeBudget));
    }

    #[test]
    fn apply_dry_run_does_not_touch_files() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("victim.bin");
        fs::write(&p, b"hello").unwrap();
        let plan = PrunePlan {
            entries: vec![PrunePlanEntry {
                path: p.clone(),
                size_bytes: 5,
                reason: PruneReason::Wipe,
            }],
        };
        let summary = apply_prune_plan(&plan, true);
        assert!(summary.dry_run);
        assert_eq!(summary.removed, 1);
        assert_eq!(summary.bytes_freed, 5);
        assert!(p.exists(), "dry-run must not delete file");
    }

    #[test]
    fn apply_real_run_removes_files() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("victim.bin");
        fs::write(&p, b"hello").unwrap();
        let plan = PrunePlan {
            entries: vec![PrunePlanEntry {
                path: p.clone(),
                size_bytes: 5,
                reason: PruneReason::Wipe,
            }],
        };
        let summary = apply_prune_plan(&plan, false);
        assert_eq!(summary.removed, 1);
        assert_eq!(summary.bytes_freed, 5);
        assert!(summary.failures.is_empty());
        assert!(!p.exists());
    }

    #[test]
    fn apply_records_failures_without_aborting() {
        let plan = PrunePlan {
            entries: vec![PrunePlanEntry {
                path: PathBuf::from("/path/does/not/exist.bin"),
                size_bytes: 0,
                reason: PruneReason::Wipe,
            }],
        };
        let summary = apply_prune_plan(&plan, false);
        assert_eq!(summary.removed, 0);
        assert_eq!(summary.failures.len(), 1);
    }

    #[test]
    fn collapse_empty_dirs_removes_empty_subtree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let deep = root.join("metadata/empty-pkg/v1");
        fs::create_dir_all(&deep).unwrap();
        let removed = collapse_empty_dirs(root);
        assert!(removed >= 1);
        assert!(!deep.exists());
        // Root remains.
        assert!(root.exists());
    }

    #[test]
    fn collapse_keeps_non_empty_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let keep = root.join("metadata/keep-pkg");
        fs::create_dir_all(&keep).unwrap();
        fs::write(keep.join("json-api.json"), b"{}").unwrap();
        let _ = collapse_empty_dirs(root);
        assert!(keep.exists());
        assert!(keep.join("json-api.json").exists());
    }

    #[test]
    fn end_to_end_clean_all_through_inventory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(&root.join("metadata/r/json-api.json"), 100);
        touch(&root.join("artifacts/r/x.whl"), 500);
        let inv = enumerate_cache(root).unwrap();
        assert_eq!(inv.count(), 2);
        let plan = plan_prune(&inv, &PrunePolicy::clean_all(), SystemTime::now());
        assert_eq!(plan.len(), 2);
        let summary = apply_prune_plan(&plan, false);
        assert_eq!(summary.removed, 2);
        let inv2 = enumerate_cache(root).unwrap();
        assert_eq!(inv2.count(), 0);
    }
}
