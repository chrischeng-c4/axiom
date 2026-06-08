// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Parallel worker pool and shard partitioner for the native test runner.
//!
//! See `.aw/tech-design/projects/jet/logic/worker-pool.md`.
//!
//! # Components
//!
//! - [`ShardSpec`] — parsed `--shard=i/N` configuration (1-indexed).
//! - [`partition_shard`] — deterministic SHA-256 hash-based partition.
//! - [`WorkerPool::run`] — bounded-concurrency tokio task pool.

use crate::test_runner::config::RunnerConfig;
use crate::test_runner::discovery::SpecFile;
use crate::test_runner::reporter::{MultiReporter, Outcome, Summary, TestError, TestReport};
use crate::test_runner::worker;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// GH #3616 — the prior shard-key computation was
/// `path.canonicalize().unwrap_or_else(|_| path.clone())`. When
/// canonicalize() failed (relative path with non-existent ancestor, EACCES
/// on a parent, symlink loop) the fallback silently used the non-canonical
/// path. The whole point of canonicalizing before hashing is that shard
/// membership is a stable function of the file, not the current working
/// directory — the silent fallback violated that contract.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn safe_shard_key(path: &Path) -> Result<String, String> {
    match path.canonicalize() {
        Ok(abs) => match abs.to_str() {
            // GH #3757 — refuse to lossy-encode the canonical path into
            // the shard key. Two distinct on-disk paths whose canonical
            // forms differ only in non-UTF-8 bytes would otherwise be
            // U+FFFD-substituted into the same key and assigned to the
            // same shard bucket, breaking the determinism contract.
            Some(s) => Ok(s.to_string()),
            None => Err(format_safe_shard_key_non_utf8_err(&abs)),
        },
        Err(e) => Err(format_safe_shard_key_err(path, &e)),
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_safe_shard_key_err(path: &Path, err: &std::io::Error) -> String {
    format!(
        "GH #3616 jet test cannot canonicalize {:?} for shard hashing ({}); \
         shard assignment must be a stable function of the file, refusing \
         to ship a non-deterministic bucket.",
        path, err
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_safe_shard_key_non_utf8_err(abs: &Path) -> String {
    format!(
        "GH #3757 jet test cannot hash canonical path {:?} for shard bucketing \
         because it is not valid UTF-8; refusing silent U+FFFD substitution \
         that would collide distinct on-disk paths into the same shard.",
        abs
    )
}

/// Shard selection: `(i, N)` where `i` is 1-indexed and `N` is total shards.
///
/// Parsed from the CLI `--shard=i/N` string.
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
pub type ShardSpec = Option<(u32, u32)>;

/// Parse a shard string of the form `"i/N"` into a `(u32, u32)` tuple.
///
/// Returns `Err` if the string is malformed or if `i > N`, `i < 1`, or `N < 1`.
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
pub fn parse_shard(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(format!("invalid --shard format: expected i/N, got {:?}", s));
    }
    let i: u32 = parts[0]
        .parse()
        .map_err(|_| format!("invalid shard index {:?}", parts[0]))?;
    let n: u32 = parts[1]
        .parse()
        .map_err(|_| format!("invalid shard total {:?}", parts[1]))?;
    if n < 1 {
        return Err("shard total N must be >= 1".to_string());
    }
    if i < 1 {
        return Err("shard index i must be >= 1".to_string());
    }
    if i > n {
        return Err(format!("shard index {} exceeds total {}", i, n));
    }
    Ok((i, n))
}

/// Partition `specs` into shard `i` of `N` using a deterministic SHA-256 hash
/// of each spec's absolute path.
///
/// Bucket formula: `sha256(absolute_path) as u64 % N == (i - 1)`.
///
/// When `shard` is `None`, all specs are returned unchanged.
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R4
pub fn partition_shard(specs: &[SpecFile], shard: ShardSpec) -> anyhow::Result<Vec<SpecFile>> {
    let (i, n) = match shard {
        None => return Ok(specs.to_vec()),
        Some(s) => s,
    };

    let target_bucket = (i - 1) as u64;

    let mut selected = Vec::new();
    for spec in specs {
        let key = safe_shard_key(&spec.path).map_err(anyhow::Error::msg)?;
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let digest = hasher.finalize();
        // Take first 8 bytes as little-endian u64
        let hash_u64 = u64::from_le_bytes(digest[..8].try_into().unwrap());
        if hash_u64 % (n as u64) == target_bucket {
            selected.push(spec.clone());
        }
    }
    Ok(selected)
}

/// Compute the SHA-256 hash of a path string (for use in tests / tracing).
///
/// Returns the hash as a u64 (first 8 bytes, little-endian).
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R4
pub fn path_hash_u64(path: &PathBuf) -> u64 {
    // GH #3616 — non-canonical fallback intentionally tolerated here:
    // this helper is only used by tests/tracing and predates the
    // determinism contract enforced in `partition_shard`. The shard
    // path (which actually decides bucket membership) refuses the
    // fallback via `safe_shard_key`.
    let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
    let abs_str = abs.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(abs_str.as_bytes());
    let digest = hasher.finalize();
    u64::from_le_bytes(digest[..8].try_into().unwrap())
}

/// Bounded-concurrency worker pool for parallel spec execution.
///
/// Uses a tokio `Semaphore` with `workers` permits to bound concurrency.
/// Each task runs one spec file to completion in its own tokio task.
/// Crashed tasks surface as errored `TestReport`s; the pool never halts.
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R5
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R6
pub struct WorkerPool;

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl WorkerPool {
    /// Run `specs` with up to `workers` concurrent tasks, each using its own
    /// browser context. Returns a merged `Summary` of all results.
    ///
    /// When `workers == 1`, specs are run serially without spawning tasks (R2).
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R2
    pub async fn run(
        specs: Vec<SpecFile>,
        workers: usize,
        config: RunnerConfig,
        reporter: Arc<MultiReporter>,
    ) -> Summary {
        if workers <= 1 {
            // Serial path — identical behavior to the existing loop (R2).
            return Self::run_serial(specs, config, reporter).await;
        }

        let semaphore = Arc::new(Semaphore::new(workers));
        let config = Arc::new(config);
        let mut task_set = tokio::task::JoinSet::new();

        for spec in specs {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore closed");
            let cfg = config.clone();
            let rep = reporter.clone();
            task_set.spawn(async move {
                let result = worker::run_spec(&spec, &cfg, &rep).await;
                // Permit released when this closure drops it.
                drop(permit);
                (spec, result)
            });
        }

        let mut summary = Summary::default();
        while let Some(join_result) = task_set.join_next().await {
            match join_result {
                Ok((_, Ok(file_summary))) => {
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R5
                    summary.passed += file_summary.passed;
                    summary.failed += file_summary.failed;
                    summary.skipped += file_summary.skipped;
                    summary.duration_ms += file_summary.duration_ms;
                    summary.reports.extend(file_summary.reports);
                    summary
                        .browser_sessions
                        .extend(file_summary.browser_sessions);
                }
                Ok((spec, Err(err))) => {
                    // Worker error (non-panic): surface as errored test.
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R5
                    summary.failed += 1;
                    summary.reports.push(TestReport {
                        file: spec.path.clone(),
                        suite: Vec::new(),
                        name: "<worker crash>".to_string(),
                        outcome: Outcome::Crashed,
                        duration_ms: 0,
                        error: Some(TestError {
                            message: format!("{err:#}"),
                            stack: None,
                            diff: None,
                            source_location: None,
                        }),
                        trace_path: None,
                        shard_index: None,
                        shard_total: None,
                        artifacts: Vec::new(),
                    });
                }
                Err(join_err) => {
                    // Task panicked — pool continues (R5).
                    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R5
                    summary.failed += 1;
                    summary.reports.push(TestReport {
                        file: PathBuf::from("<unknown>"),
                        suite: Vec::new(),
                        name: "<worker panic>".to_string(),
                        outcome: Outcome::Crashed,
                        duration_ms: 0,
                        error: Some(TestError {
                            message: format!("worker task panicked: {join_err}"),
                            stack: None,
                            diff: None,
                            source_location: None,
                        }),
                        trace_path: None,
                        shard_index: None,
                        shard_total: None,
                        artifacts: Vec::new(),
                    });
                }
            }
        }

        summary
    }

    /// Serial execution (workers == 1) — matches Phase 1-3 behavior exactly.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R2
    async fn run_serial(
        specs: Vec<SpecFile>,
        config: RunnerConfig,
        reporter: Arc<MultiReporter>,
    ) -> Summary {
        let mut summary = Summary::default();
        for spec in &specs {
            match worker::run_spec(spec, &config, &reporter).await {
                Ok(file_summary) => {
                    summary.passed += file_summary.passed;
                    summary.failed += file_summary.failed;
                    summary.skipped += file_summary.skipped;
                    summary.duration_ms += file_summary.duration_ms;
                    summary.reports.extend(file_summary.reports);
                    summary
                        .browser_sessions
                        .extend(file_summary.browser_sessions);
                }
                Err(err) => {
                    summary.failed += 1;
                    summary.reports.push(TestReport {
                        file: spec.path.clone(),
                        suite: Vec::new(),
                        name: "<worker crash>".to_string(),
                        outcome: Outcome::Crashed,
                        duration_ms: 0,
                        error: Some(TestError {
                            message: format!("{err:#}"),
                            stack: None,
                            diff: None,
                            source_location: None,
                        }),
                        trace_path: None,
                        shard_index: None,
                        shard_total: None,
                        artifacts: Vec::new(),
                    });
                }
            }
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // GH #3616 — make_spec_on_disk creates a real file so canonicalize()
    // succeeds in partition_shard. Pre-#3616 tests used phantom /tmp paths;
    // those would now fail the canonicalize gate.
    fn make_spec_on_disk(dir: &Path, name: &str) -> SpecFile {
        let path = dir.join(name);
        std::fs::write(&path, "").expect("write test spec stub");
        SpecFile {
            path: path.clone(),
            relative: PathBuf::from(name),
        }
    }

    #[test]
    fn parse_shard_valid() {
        assert_eq!(parse_shard("1/4").unwrap(), (1, 4));
        assert_eq!(parse_shard("2/4").unwrap(), (2, 4));
        assert_eq!(parse_shard("4/4").unwrap(), (4, 4));
    }

    #[test]
    fn parse_shard_invalid() {
        assert!(parse_shard("0/4").is_err());
        assert!(parse_shard("5/4").is_err());
        assert!(parse_shard("abc").is_err());
        assert!(parse_shard("1/0").is_err());
        assert!(parse_shard("").is_err());
    }

    #[test]
    fn partition_none_returns_all() {
        let dir = tempfile::tempdir().unwrap();
        let specs: Vec<SpecFile> = (0..5)
            .map(|i| make_spec_on_disk(dir.path(), &format!("s{}.spec.ts", i)))
            .collect();
        let result = partition_shard(&specs, None).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn partition_shard_1_of_1_returns_all() {
        let dir = tempfile::tempdir().unwrap();
        let specs: Vec<SpecFile> = (0..5)
            .map(|i| make_spec_on_disk(dir.path(), &format!("s{}.spec.ts", i)))
            .collect();
        let result = partition_shard(&specs, Some((1, 1))).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn partition_shard_membership_is_stable_for_fixed_manifest() {
        // @spec #2867 — stop condition: a manifest fixture proves
        // stable shard membership. Same input manifest + same (i, N)
        // must yield byte-identical output across invocations, so
        // CI can split work deterministically.
        let dir = tempfile::tempdir().unwrap();
        let specs: Vec<SpecFile> = (0..16)
            .map(|i| make_spec_on_disk(dir.path(), &format!("spec_{:02}.spec.ts", i)))
            .collect();
        let n = 4u32;
        for i in 1..=n {
            let a = partition_shard(&specs, Some((i, n))).unwrap();
            let b = partition_shard(&specs, Some((i, n))).unwrap();
            assert_eq!(
                a.iter().map(|s| s.path.clone()).collect::<Vec<_>>(),
                b.iter().map(|s| s.path.clone()).collect::<Vec<_>>(),
                "shard {i}/{n} membership must be stable across calls",
            );
        }
    }

    #[test]
    fn parse_shard_errors_are_human_readable() {
        // @spec #2867 — invalid shard values must return clear errors
        // (not just generic parse failures) so CI scripts can surface
        // the cause to a human.
        let cases = [
            ("0/4", "shard index i must be >= 1"),
            ("5/4", "exceeds total"),
            ("1/0", "shard total N must be >= 1"),
            ("abc", "invalid --shard format"),
            ("", "invalid --shard format"),
        ];
        for (input, expect_substring) in cases {
            let err = parse_shard(input).unwrap_err();
            assert!(
                err.contains(expect_substring),
                "input {input:?}: expected error containing {expect_substring:?}, got {err:?}",
            );
        }
    }

    #[test]
    fn partition_shards_are_disjoint_and_cover_all() {
        let dir = tempfile::tempdir().unwrap();
        let specs: Vec<SpecFile> = (0..12)
            .map(|i| make_spec_on_disk(dir.path(), &format!("spec_{}.spec.ts", i)))
            .collect();
        let n = 4u32;
        let mut all_paths: Vec<PathBuf> = Vec::new();
        for i in 1..=n {
            let shard = partition_shard(&specs, Some((i, n))).unwrap();
            for s in &shard {
                assert!(!all_paths.contains(&s.path), "duplicate in shard {}", i);
                all_paths.push(s.path.clone());
            }
        }
        // All specs should be covered across all shards
        assert_eq!(all_paths.len(), specs.len());
    }
}

#[cfg(test)]
mod gh3616_safe_shard_key_tests {
    //! GH #3616 — partition_shard previously did
    //! `path.canonicalize().unwrap_or_else(|_| path.clone())`. When
    //! canonicalize() failed, the silent fallback used the non-canonical
    //! path, defeating the canonicalize purpose: shard membership was
    //! supposed to be a stable function of the file, not the CWD.
    use super::*;
    use crate::test_runner::discovery::SpecFile;
    use std::path::PathBuf;

    #[test]
    fn existing_file_returns_canonical_string() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.spec.ts");
        std::fs::write(&p, "").unwrap();
        let key = safe_shard_key(&p).expect("existing file canonicalizes");
        let want = p.canonicalize().unwrap().to_string_lossy().into_owned();
        assert_eq!(key, want);
    }

    #[test]
    fn nonexistent_path_returns_tagged_err() {
        let p = PathBuf::from("/nonexistent/jet-gh3616/should-fail.spec.ts");
        let err = safe_shard_key(&p).expect_err("nonexistent path must not silently fall back");
        assert!(err.contains("GH #3616"), "err: {err}");
        assert!(err.contains("should-fail.spec.ts"), "err: {err}");
    }

    #[test]
    fn helper_message_includes_tag_and_path() {
        let p = PathBuf::from("/x/y.ts");
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let msg = format_safe_shard_key_err(&p, &io_err);
        assert!(msg.contains("GH #3616"), "msg: {msg}");
        assert!(msg.contains("/x/y.ts"), "msg: {msg}");
        assert!(msg.contains("no such file"), "msg: {msg}");
    }

    #[test]
    fn partition_shard_refuses_nonexistent_spec() {
        let bad = SpecFile {
            path: PathBuf::from("/nonexistent/gh3616/missing.spec.ts"),
            relative: PathBuf::from("missing.spec.ts"),
        };
        let result = partition_shard(&[bad], Some((1, 2)));
        assert!(result.is_err(), "must propagate canonicalize failure");
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("GH #3616"), "err: {err}");
    }

    #[test]
    fn partition_shard_none_skips_canonicalize_even_for_nonexistent() {
        // shard == None is a no-op early return: it should NOT exercise
        // the canonicalize gate, so non-existent paths are still valid
        // input when sharding is disabled.
        let bad = SpecFile {
            path: PathBuf::from("/nonexistent/gh3616/missing.spec.ts"),
            relative: PathBuf::from("missing.spec.ts"),
        };
        let result = partition_shard(&[bad], None).expect("None shard short-circuits");
        assert_eq!(result.len(), 1);
    }
}

/// GH #3757 — silent lossy UTF-8 substitution in `safe_shard_key`.
///
/// The fix wraps the canonical path's string conversion in a
/// `to_str().ok_or_else(...)` rejection, surfaced via the new
/// `format_safe_shard_key_non_utf8_err` helper. These tests pin the
/// helper contract and confirm the wrapper still passes valid UTF-8
/// through (so the #3616 happy path is preserved).
#[cfg(test)]
mod gh3757_safe_shard_key_non_utf8_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn gh3757_non_utf8_err_carries_issue_tag_and_path() {
        let p = PathBuf::from("/tmp/jet-gh3757/weird.ts");
        let msg = format_safe_shard_key_non_utf8_err(&p);
        assert!(msg.contains("GH #3757"), "issue tag missing: {msg}");
        assert!(msg.contains("not valid UTF-8"), "shape missing: {msg}");
        assert!(msg.contains("weird.ts"), "path missing: {msg}");
    }

    #[test]
    fn gh3757_non_utf8_err_is_deterministic() {
        let p = PathBuf::from("/a.ts");
        let a = format_safe_shard_key_non_utf8_err(&p);
        let b = format_safe_shard_key_non_utf8_err(&p);
        assert_eq!(a, b);
    }

    #[test]
    fn gh3757_distinct_paths_produce_distinct_messages() {
        let a = format_safe_shard_key_non_utf8_err(&PathBuf::from("/a.ts"));
        let b = format_safe_shard_key_non_utf8_err(&PathBuf::from("/b.ts"));
        assert_ne!(a, b);
    }

    #[test]
    fn gh3757_non_utf8_err_is_distinct_from_canonicalize_err() {
        // The sibling `format_safe_shard_key_err` covers canonicalize
        // failures with the "GH #3616" tag; our helper must carry its
        // own tag so operators can grep the two distinct failure modes.
        let canon_msg = format_safe_shard_key_err(
            &PathBuf::from("/x.ts"),
            &std::io::Error::new(std::io::ErrorKind::NotFound, "nope"),
        );
        let utf8_msg = format_safe_shard_key_non_utf8_err(&PathBuf::from("/x.ts"));
        assert!(canon_msg.contains("GH #3616"));
        assert!(utf8_msg.contains("GH #3757"));
        assert!(!canon_msg.contains("GH #3757"));
        assert!(!utf8_msg.contains("GH #3616"));
    }

    #[test]
    fn gh3757_valid_utf8_canonical_path_still_passes_through() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("ok.spec.ts");
        std::fs::write(&p, "").unwrap();
        let key = safe_shard_key(&p).expect("UTF-8 path canonicalizes + stringifies");
        let want = p.canonicalize().unwrap();
        // Sanity: the key now goes through `to_str()`, not `to_string_lossy()`.
        // On Unix the canonical path is UTF-8 so `to_str()` returns Some(_).
        assert_eq!(key, want.to_str().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn gh3757_non_utf8_pathbuf_to_str_returns_none() {
        // Cross-platform contract probe: a PathBuf built from non-UTF-8
        // bytes must fail `.to_str()`, which is the gate `safe_shard_key`
        // now relies on. (We don't write to disk because macOS APFS
        // refuses non-UTF-8 names; see #3753 for context.)
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let bad = PathBuf::from(OsStr::from_bytes(&[b'a', 0xFF, b'b']));
        assert!(bad.to_str().is_none());
    }

    #[test]
    fn gh3757_helper_name_follows_family_convention() {
        // Discoverability: callers searching for "format_safe_shard_key"
        // should find both `_err` (canonicalize) and `_non_utf8_err`
        // (this) siblings in this module.
        let msg = format_safe_shard_key_non_utf8_err(&PathBuf::from("/x"));
        assert!(msg.contains("shard"), "missing area anchor: {msg}");
        assert!(
            msg.contains("canonical path"),
            "missing canonical anchor: {msg}"
        );
    }

    #[test]
    fn gh3757_partition_shard_still_works_for_valid_paths() {
        use crate::test_runner::discovery::SpecFile;
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.spec.ts");
        let b = dir.path().join("b.spec.ts");
        std::fs::write(&a, "").unwrap();
        std::fs::write(&b, "").unwrap();
        let specs = vec![
            SpecFile {
                path: a,
                relative: PathBuf::from("a.spec.ts"),
            },
            SpecFile {
                path: b,
                relative: PathBuf::from("b.spec.ts"),
            },
        ];
        // Should succeed and never call the new helper.
        let _ = partition_shard(&specs, Some((1, 2))).expect("valid UTF-8 paths must shard");
    }

    #[test]
    fn gh3757_err_message_includes_canonical_quoting() {
        // `{:?}` on Path renders bytes deterministically; this is what
        // operators will see in the log. Lock in the surface so the
        // raw bytes — not a silent U+FFFD — appear.
        let p = PathBuf::from("/canon/path.ts");
        let msg = format_safe_shard_key_non_utf8_err(&p);
        assert!(
            msg.contains("\"/canon/path.ts\""),
            "expected debug-quoted path: {msg}"
        );
    }
}
// CODEGEN-END
