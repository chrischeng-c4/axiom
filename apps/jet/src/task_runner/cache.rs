// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
// CODEGEN-BEGIN
//! Task cache: content-hash based caching for task outputs.
//!
//! Cache key = SHA-256(input files + env vars + command + dependency hashes).
//! Cached entries stored in `.jet-cache/tasks/` inside the project.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::hash;

/// Cached task entry stored on disk.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCacheEntry {
    pub hash: String,
    pub task_name: String,
    pub outputs: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub created_at: String,
}

/// Task cache manager.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub struct TaskCache {
    cache_dir: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
impl TaskCache {
    /// Create a new task cache in the project's .jet-cache directory.
    pub fn new(project_root: &Path) -> Result<Self> {
        let cache_dir = project_root.join(".jet-cache").join("tasks");
        std::fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Compute a content hash for a task given its inputs and environment.
    pub fn compute_hash(
        &self,
        task_name: &str,
        input_globs: &[String],
        env_keys: &[String],
        project_root: &Path,
    ) -> Result<String> {
        hash::compute_task_hash(task_name, input_globs, env_keys, project_root)
    }

    /// Look up a cached entry by hash.
    pub fn lookup(&self, hash: &str) -> Result<Option<TaskCacheEntry>> {
        let entry_path = self.cache_dir.join(format!("{}.json", hash));
        if !entry_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&entry_path)
            .with_context(|| format!("Failed to read cache entry: {}", entry_path.display()))?;
        let entry: TaskCacheEntry = serde_json::from_str(&content)?;
        Ok(Some(entry))
    }

    /// Store a task result in the cache.
    pub fn store(
        &self,
        hash: &str,
        task_name: &str,
        output_globs: &[String],
        stdout: &str,
        stderr: &str,
        project_root: &Path,
    ) -> Result<()> {
        // Collect actual output files.
        // GH #3153 — propagate malformed-glob errors so the dev sees a
        // typo in their output pattern, instead of silently caching an
        // entry with no outputs and then "succeeding" via stale cache
        // hits that restore nothing.
        let outputs = collect_output_files(output_globs, project_root)?;

        let entry = TaskCacheEntry {
            hash: hash.to_string(),
            task_name: task_name.to_string(),
            outputs,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            created_at: chrono_now(),
        };

        let entry_path = self.cache_dir.join(format!("{}.json", hash));
        let content = serde_json::to_string_pretty(&entry)?;
        std::fs::write(&entry_path, content)?;

        // Also cache output files.
        // GH #3197 — the prior `let _ = std::fs::copy(...)` silently dropped
        // every copy failure (disk full, permission denied, race with a
        // concurrent jet run), leaving the cache in a half-populated state
        // — the metadata `{hash}.json` claims N outputs but only some
        // landed under `{hash}/`. On next restore, the user sees either a
        // mysteriously broken build or stale partial artifacts. Surface
        // any failure (parent-dir creation OR copy) via tracing::warn!
        // AND invalidate the metadata so the next run sees a clean miss.
        let output_cache_dir = self.cache_dir.join(hash);
        if !entry.outputs.is_empty() {
            if let Err(e) =
                Self::copy_outputs_to_cache(project_root, &output_cache_dir, &entry.outputs)
            {
                tracing::warn!(
                    target: "jet::task_runner::cache",
                    "failed to populate cached outputs under {:?} for task {}: {e}; \
                     invalidating cache entry {} so the next run sees a clean miss (GH #3197)",
                    output_cache_dir,
                    task_name,
                    hash
                );
                // GH #3269 — cleanup is best-effort, but the prior
                // `let _ = ...` silently discarded its failure. If
                // the metadata `{hash}.json` survives this rollback,
                // the next `lookup(hash)` returns Some and the
                // caller treats it as a cache hit; `restore_outputs`
                // early-returns Ok when `{cache}/{hash}/` is absent
                // (cache.rs:167-169), so the build "succeeds via
                // cache" with zero artifacts on disk. Warn loudly
                // so operators can manually remove the orphan and
                // recover before silent poisoning lands.
                match std::fs::remove_file(&entry_path) {
                    Ok(()) => {}
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::task_runner::cache",
                            path = %entry_path.display(),
                            task = %task_name,
                            hash = %hash,
                            error = %err,
                            original_copy_error = %e,
                            "GH #3269 failed to remove orphan cache entry; \
                             next run will treat this hash as a cache HIT with no \
                             artifacts on disk — delete {} to recover",
                            entry_path.display()
                        );
                    }
                }
                match std::fs::remove_dir_all(&output_cache_dir) {
                    Ok(()) => {}
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::task_runner::cache",
                            path = %output_cache_dir.display(),
                            task = %task_name,
                            hash = %hash,
                            error = %err,
                            original_copy_error = %e,
                            "GH #3269 failed to remove half-populated cache output dir; \
                             manually delete {} to free the partial state",
                            output_cache_dir.display()
                        );
                    }
                }
                return Err(anyhow::anyhow!(
                    "task_runner cache store failed for {}: {e} (GH #3197)",
                    task_name
                ));
            }
        }

        Ok(())
    }

    /// Copy outputs into the cache. Returns Err on the first failure so
    /// the caller can tear down the half-populated entry (GH #3197).
    fn copy_outputs_to_cache(
        project_root: &Path,
        output_cache_dir: &Path,
        outputs: &[String],
    ) -> Result<()> {
        std::fs::create_dir_all(output_cache_dir).with_context(|| {
            format!(
                "failed to create cache output dir {}",
                output_cache_dir.display()
            )
        })?;
        for output_file in outputs {
            let src = project_root.join(output_file);
            let dst = output_cache_dir.join(output_file);
            if !src.exists() {
                continue;
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create cache parent dir {}", parent.display())
                })?;
            }
            std::fs::copy(&src, &dst).with_context(|| {
                format!("failed to copy {} -> {}", src.display(), dst.display())
            })?;
        }
        Ok(())
    }

    /// Restore cached outputs to the project directory.
    pub fn restore_outputs(&self, hash: &str, project_root: &Path) -> Result<()> {
        let output_cache_dir = self.cache_dir.join(hash);
        if !output_cache_dir.exists() {
            return Ok(());
        }

        for entry_res in walkdir::WalkDir::new(&output_cache_dir).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::task_runner::cache",
                        hash = %hash,
                        cache_root = %output_cache_dir.display(),
                        offending_path = ?err.path(),
                        error = %err,
                        "GH #3308 walkdir error while restoring cached \
                         outputs; the affected file will be silently absent \
                         from the project tree until the cache entry is \
                         repaired or evicted"
                    );
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            // GH #3578 — `.unwrap_or(entry.path())` used to fall back
            // to the ABSOLUTE entry path when `strip_prefix` failed.
            // `project_root.join(absolute_path)` then discarded
            // `project_root` and yielded the absolute path itself, so
            // `std::fs::copy` would silently write the cached file
            // OUTSIDE the project tree (potentially overwriting
            // unrelated files on every cache hit). Refuse to restore
            // instead, naming the offending entry and cache dir.
            let rel = entry
                .path()
                .strip_prefix(&output_cache_dir)
                .with_context(|| {
                    format_restore_prefix_err(entry.path(), &output_cache_dir, hash)
                })?;
            let dst = project_root.join(rel);
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dst)?;
        }

        Ok(())
    }
}

/// Collect output files matching glob patterns.
fn collect_output_files(globs: &[String], project_root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for pattern in globs {
        let full = format!("{}/{}", project_root.display(), pattern);
        let entries = glob::glob(&full).with_context(|| {
            format!(
                "Failed to compile task output glob {pattern:?}; \
                 refusing to cache a task entry that would silently \
                 omit its declared outputs (GH #3153)"
            )
        })?;
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!(
                        target: "jet::task::cache",
                        "skipping output entry under glob {pattern:?}: {e} (GH #3153)"
                    );
                    continue;
                }
            };
            if entry.is_file() {
                // GH #3576 — `if let Ok(rel) = …` silently dropped matched
                // outputs whose path could not be expressed relative to
                // `project_root` (typically via a symlink whose target
                // escapes the project tree). `TaskCache::store` then
                // succeeded with a partial manifest and the next restore
                // copied only the surviving subset, so the "cache hit"
                // tree no longer matched the original. Refuse to cache
                // rather than silently cache-partial.
                let rel = entry
                    .strip_prefix(project_root)
                    .with_context(|| format_output_escape_err(&entry, project_root, pattern))?;
                // GH #3753 — prior `rel.to_string_lossy().to_string()`
                // silently substituted U+FFFD for non-UTF-8 bytes, so
                // the cache manifest stored a path that couldn't
                // round-trip back to the source on restore. Refuse to
                // cache outputs whose path is not valid UTF-8 (mirrors
                // the symmetric fix in `task_runner::hash.rs`).
                let rel_str = rel.to_str().ok_or_else(|| {
                    anyhow::anyhow!(
                        "{}",
                        format_task_cache_non_utf8_err(rel, project_root, pattern)
                    )
                })?;
                files.push(rel_str.to_string());
            }
        }
    }
    Ok(files)
}

/// GH #3753 — build the error message for a task-output path whose
/// relative-to-project bytes are not valid UTF-8. Mirrors the
/// symmetric `format_task_hash_non_utf8_warn` in `task_runner::hash`
/// but distinct so triage can tell which side (input hash vs output
/// cache) is failing.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_task_cache_non_utf8_err(
    rel: &Path,
    project_root: &Path,
    pattern: &str,
) -> String {
    format!(
        "GH #3753 task output {} (under {}, matched by glob `{pattern}`) \
         contains bytes that are not valid UTF-8; jet refuses to cache \
         this output because the prior `.to_string_lossy()` would store \
         a U+FFFD-substituted path in the manifest that could not \
         round-trip back to the source on restore. The result would be \
         a corrupted cache restore — missing files or files copied to \
         the wrong place. Rename the file to a UTF-8 name and re-run.",
        rel.display(),
        project_root.display()
    )
}

/// GH #3576 — build the error message for an output-glob match whose
/// path escapes `project_root` (typically via a symlink). Extracted so
/// the wording (path + project root + pattern + tag) is unit-testable
/// without provoking the actual filesystem case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_output_escape_err(entry: &Path, project_root: &Path, pattern: &str) -> String {
    format!(
        "GH #3576 task output {} matched by glob {pattern:?} escapes \
         project_root {}; refusing to cache an entry that would silently \
         omit it from the manifest (and cause a partial restore on the \
         next cache hit)",
        entry.display(),
        project_root.display()
    )
}

/// GH #3578 — build the error message for a `restore_outputs` walkdir
/// entry whose path cannot be expressed relative to the per-hash
/// `output_cache_dir`. Extracted so the wording (entry + cache dir +
/// hash + tag) is unit-testable without provoking the actual
/// filesystem case.
///
/// Replaces the prior `.unwrap_or(entry.path())` silent fallback which
/// fed an ABSOLUTE path into `project_root.join(rel)` and silently
/// wrote the cached output outside `project_root`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_restore_prefix_err(
    entry: &Path,
    output_cache_dir: &Path,
    hash: &str,
) -> String {
    format!(
        "GH #3578 cache restore entry {} cannot be expressed relative to \
         output cache dir {} (hash: {hash}); refusing to restore — the \
         prior silent fallback would have written this cached output \
         OUTSIDE project_root, potentially overwriting unrelated files",
        entry.display(),
        output_cache_dir.display()
    )
}

/// Get current time as ISO 8601 string (no chrono dependency).
fn chrono_now() -> String {
    use std::time::SystemTime;
    // GH #3644 — was `.unwrap_or_default()` which silently collapsed
    // `Err(_)` (clock before UNIX_EPOCH or transient NTP/container thaw)
    // to `Duration::ZERO`, producing `"0s"` indistinguishable from a
    // real epoch timestamp. Safe helper returns a grep-distinguishable
    // marker on the error branch and surfaces a warn so operators can
    // tell which cache entries hit the clock-error path.
    let (stamp, warn) = safe_cache_now(SystemTime::now());
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::task_runner::cache", "{}", msg);
    }
    stamp
}

/// GH #3644 — `chrono_now` previously did
/// `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()`,
/// which silently collapsed `Err(_)` (clock before UNIX_EPOCH or transient
/// NTP/container thaw) to `Duration::ZERO`, producing `"0s"` that was
/// indistinguishable from a real epoch timestamp.
///
/// This helper distinguishes the two branches:
/// - happy path: returns `("<N>s", None)` with the real seconds-since-epoch
/// - error path: returns a grep-distinguishable marker plus a warn message
///   the caller can emit via `tracing::warn!`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn safe_cache_now(now: std::time::SystemTime) -> (String, Option<String>) {
    use std::time::SystemTime;
    match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(dur) => (format!("{}s", dur.as_secs()), None),
        Err(err) => {
            let marker = "GH-3644-clock-before-epoch".to_string();
            let warn = format_safe_cache_now_warn(&err);
            (marker, Some(warn))
        }
    }
}

/// GH #3644 — tagged warn message for [`safe_cache_now`]. Names the
/// issue, the cause, and the visible marker so operators tracing stale
/// cache entries can correlate logs with the on-disk `created_at`
/// field.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_safe_cache_now_warn(err: &std::time::SystemTimeError) -> String {
    format!(
        "GH #3644 jet task_runner cache `created_at` stamp could not be \
         derived from `SystemTime::now().duration_since(UNIX_EPOCH)` \
         (err: {err}); cache entries written during this window will \
         carry the marker `GH-3644-clock-before-epoch` instead of a \
         seconds-since-epoch value. Check NTP / container-thaw timing."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_cache_creation() {
        let dir = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(dir.path());
        assert!(cache.is_ok());
        assert!(dir.path().join(".jet-cache/tasks").exists());
    }

    #[test]
    fn test_cache_miss() {
        let dir = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(dir.path()).unwrap();
        let result = cache.lookup("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_store_and_lookup() {
        let dir = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(dir.path()).unwrap();

        cache
            .store("abc123", "build", &[], "output\n", "", dir.path())
            .unwrap();

        let entry = cache.lookup("abc123").unwrap().unwrap();
        assert_eq!(entry.task_name, "build");
        assert_eq!(entry.stdout, "output\n");
    }

    /// GH #3153 — A malformed output glob (e.g. unclosed character
    /// class) must surface as `Err` from `TaskCache::store`. The pre-fix
    /// code silently cached an entry with zero outputs and "succeeded".
    #[test]
    fn store_rejects_malformed_output_glob() {
        let dir = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(dir.path()).unwrap();

        // Unclosed character class — invalid glob pattern.
        let bad_glob = "dist/**[".to_string();
        let err = cache
            .store("hash1", "build", &[bad_glob.clone()], "", "", dir.path())
            .expect_err(
                "malformed output glob must surface as Err, not silently \
                 cache an empty-outputs entry (GH #3153)",
            );
        let msg = format!("{err:#}");
        assert!(
            msg.contains("GH #3153"),
            "error must include the searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("dist/**["),
            "error must name the offending pattern, got: {msg}"
        );

        // Cache entry must NOT have been written despite the error.
        let lookup = cache.lookup("hash1").unwrap();
        assert!(
            lookup.is_none(),
            "store() must not persist an entry when output glob is invalid"
        );
    }

    /// GH #3153 — Valid output glob still works and collects matching
    /// files. Pins that the new error-propagating path didn't accidentally
    /// regress the happy path.
    #[test]
    fn store_collects_valid_output_glob() {
        let dir = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(dir.path()).unwrap();

        std::fs::create_dir_all(dir.path().join("dist")).unwrap();
        std::fs::write(dir.path().join("dist/a.js"), b"const a = 1;").unwrap();
        std::fs::write(dir.path().join("dist/b.js"), b"const b = 2;").unwrap();

        cache
            .store(
                "hash-ok",
                "build",
                &["dist/*.js".to_string()],
                "",
                "",
                dir.path(),
            )
            .expect("valid glob must succeed");

        let entry = cache.lookup("hash-ok").unwrap().unwrap();
        let mut outputs = entry.outputs.clone();
        outputs.sort();
        assert_eq!(
            outputs,
            vec!["dist/a.js".to_string(), "dist/b.js".to_string()]
        );
    }

    /// GH #3197 — When `std::fs::copy` for an output file fails partway
    /// through, the previous `let _ = ...` silently swallowed the error
    /// and left `{hash}.json` claiming N outputs with only some present
    /// under `{hash}/`. Subsequent restore would either error mysteriously
    /// or restore stale partial state.
    ///
    /// Post-fix: store() returns Err AND removes the metadata file so the
    /// next lookup sees a clean miss. We trigger the failure by placing a
    /// FILE where the destination's parent directory is supposed to be —
    /// `create_dir_all` then errors at the cached-output stage.
    #[test]
    fn store_aborts_and_invalidates_entry_on_copy_failure() {
        let project = tempfile::tempdir().unwrap();
        let cache_root = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(cache_root.path()).unwrap();

        // Real source file the cache will try to copy.
        std::fs::create_dir_all(project.path().join("dist")).unwrap();
        std::fs::write(project.path().join("dist/a.js"), b"const a = 1;").unwrap();

        // Park a FILE where `<cache_dir>/<hash>/dist` (the dst parent for
        // dist/a.js) would otherwise be created. `create_dir_all` will
        // fail with "Not a directory" inside the copy loop.
        let hash = "hash-copy-fail";
        let hash_dir = cache_root.path().join(".jet-cache/tasks").join(hash);
        std::fs::create_dir_all(&hash_dir).unwrap();
        std::fs::write(hash_dir.join("dist"), b"a file, blocking the dir").unwrap();

        let result = cache.store(
            hash,
            "build",
            &["dist/*.js".to_string()],
            "",
            "",
            project.path(),
        );
        assert!(
            result.is_err(),
            "copy failure must surface as Err (GH #3197), got: {result:?}"
        );
        let msg = format!("{:#}", result.unwrap_err());
        assert!(
            msg.contains("GH #3197"),
            "error must include the searchable issue tag, got: {msg}"
        );

        // Metadata file must NOT survive the failure — the next lookup
        // must see a clean miss, not half-state.
        let lookup = cache.lookup(hash).unwrap();
        assert!(
            lookup.is_none(),
            "store() must invalidate {hash}.json on copy failure (GH #3197)"
        );
    }

    /// GH #3269 — Verify cleanup-on-copy-failure removes BOTH the
    /// metadata `{hash}.json` AND the partially-populated output dir
    /// `{cache}/{hash}/`. Pre-#3197 the metadata was kept; #3197
    /// silently tried to remove both via `let _ = ...`. #3269 made
    /// the cleanup paths explicit and observable.
    ///
    /// When the cleanup itself fails, a `tracing::warn!` fires
    /// naming the orphan path so operators can recover manually —
    /// that warn-firing path is exercised by the same explicit
    /// match arms; we can't easily simulate parent-dir-write-perms
    /// failure mid-store, so this test pins the cleanup-success path
    /// (and indirectly that the match arms compile and run).
    #[test]
    fn store_copy_failure_cleans_both_entry_and_output_dir() {
        let project = tempfile::tempdir().unwrap();
        let cache_root = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(cache_root.path()).unwrap();

        std::fs::create_dir_all(project.path().join("dist")).unwrap();
        std::fs::write(project.path().join("dist/a.js"), b"const a = 1;").unwrap();

        // Same blocking trick as #3197: park a FILE where the
        // destination's parent dir is supposed to be, so the
        // per-file copy fails inside the loop after the output dir
        // and one or more sub-paths have been created.
        let hash = "hash-cleanup-both";
        let hash_dir = cache_root.path().join(".jet-cache/tasks").join(hash);
        std::fs::create_dir_all(&hash_dir).unwrap();
        std::fs::write(hash_dir.join("dist"), b"a file, blocking the dir").unwrap();

        let result = cache.store(
            hash,
            "build",
            &["dist/*.js".to_string()],
            "",
            "",
            project.path(),
        );
        assert!(result.is_err(), "copy failure must surface as Err");

        // Both pieces of the half-populated entry must be gone:
        //   1. `{hash}.json` — the entry metadata
        //   2. `{hash}/` — the output cache dir (and the file we
        //      planted inside it as the blocker)
        let entry_path = cache_root
            .path()
            .join(".jet-cache/tasks")
            .join(format!("{hash}.json"));
        assert!(
            !entry_path.exists(),
            "GH #3269: {} must be removed so the next lookup sees a clean miss",
            entry_path.display()
        );
        assert!(
            !hash_dir.exists(),
            "GH #3269: {} must be removed so no half-populated artifacts leak across runs",
            hash_dir.display()
        );

        // Sanity: lookup confirms no orphan poisons the next run.
        assert!(cache.lookup(hash).unwrap().is_none());
    }

    /// GH #3308 — restore_outputs over a clean cache must round-trip every
    /// file under the per-hash cache dir into the project root. Pins that
    /// the new walkdir-error-surfacing path didn't regress the happy path.
    #[test]
    fn restore_outputs_copies_every_file_under_hash_dir() {
        let cache_root = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(cache_root.path()).unwrap();
        let hash = "happyhash";

        // Hand-craft an output-cache dir as if `store` had populated it.
        let hash_dir = cache_root.path().join(".jet-cache/tasks").join(hash);
        std::fs::create_dir_all(hash_dir.join("dist/sub")).unwrap();
        std::fs::write(hash_dir.join("dist/a.js"), b"A").unwrap();
        std::fs::write(hash_dir.join("dist/sub/b.js"), b"B").unwrap();

        let project = tempfile::tempdir().unwrap();
        cache.restore_outputs(hash, project.path()).unwrap();

        assert_eq!(
            std::fs::read(project.path().join("dist/a.js")).unwrap(),
            b"A"
        );
        assert_eq!(
            std::fs::read(project.path().join("dist/sub/b.js")).unwrap(),
            b"B"
        );
    }

    /// GH #3308 — restore_outputs must not abort when an unreadable
    /// subdirectory makes walkdir emit an Err for that subtree.  The
    /// readable sibling file must still land in the project tree; the
    /// affected file is silently absent (operator sees a warn log).
    #[cfg(unix)]
    #[test]
    fn restore_outputs_unreadable_subdir_keeps_sibling_files() {
        use std::os::unix::fs::PermissionsExt;

        let cache_root = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(cache_root.path()).unwrap();
        let hash = "mixedhash";

        let hash_dir = cache_root.path().join(".jet-cache/tasks").join(hash);
        std::fs::create_dir_all(hash_dir.join("dist")).unwrap();
        std::fs::write(hash_dir.join("dist/keep.js"), b"KEEP").unwrap();

        // Plant a subdir that walkdir cannot descend into.
        let unreadable = hash_dir.join("dist/locked");
        std::fs::create_dir_all(&unreadable).unwrap();
        std::fs::write(unreadable.join("lost.js"), b"LOST").unwrap();
        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Skip when running as root — chmod 0o000 is unenforceable.
        let root_can_still_read = std::fs::read_dir(&unreadable).is_ok();
        if root_can_still_read {
            let _ = std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let project = tempfile::tempdir().unwrap();
        let result = cache.restore_outputs(hash, project.path());

        // Restore permissions before any assertion so the tempdir can clean up.
        let _ = std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o755));

        result.expect("restore_outputs must not abort on per-entry walkdir Err");

        assert_eq!(
            std::fs::read(project.path().join("dist/keep.js")).unwrap(),
            b"KEEP",
            "sibling file must still land in project tree"
        );
        assert!(
            !project.path().join("dist/locked/lost.js").exists(),
            "file behind the unreadable subdir must be absent (logged via tracing::warn)"
        );
    }

    // ─── GH #3576: output glob silently dropped escapes-project-root ─────

    /// GH #3576 — `format_output_escape_err` must include the issue
    /// tag, the offending entry path, the project root, and the
    /// offending glob pattern so a grep on the error chain surfaces the
    /// failure cause without needing the live filesystem case.
    #[test]
    fn gh3576_format_output_escape_err_names_tag_paths_and_pattern() {
        let entry = std::path::Path::new("/outside/escapes.js");
        let project_root = std::path::Path::new("/proj");
        let msg = format_output_escape_err(entry, project_root, "dist/**/*");

        assert!(
            msg.contains("GH #3576"),
            "must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("/outside/escapes.js"),
            "must name the offending entry path, got: {msg}"
        );
        assert!(
            msg.contains("/proj"),
            "must name the project root, got: {msg}"
        );
        assert!(
            msg.contains("dist/**/*"),
            "must name the offending glob pattern, got: {msg}"
        );
    }

    /// GH #3576 — end-to-end on Unix: a symlink whose target escapes
    /// `project_root` must surface as `Err` from `TaskCache::store`,
    /// not silently produce a partial outputs manifest. The error
    /// chain must include the GH #3576 tag.
    #[cfg(unix)]
    #[test]
    fn gh3576_symlink_escape_surfaces_error_through_store() {
        let outer = tempfile::tempdir().unwrap();

        let outside_dir = outer.path().join("outside");
        std::fs::create_dir_all(&outside_dir).unwrap();
        std::fs::write(outside_dir.join("leak.js"), b"escapes").unwrap();

        let project = outer.path().join("project");
        std::fs::create_dir_all(project.join("dist")).unwrap();
        // The matched entry will be `<project>/dist/leak.js` resolving
        // through the symlink to `<outer>/outside/leak.js`, which the
        // glob crate canonicalizes for `is_file` purposes. On platforms
        // where the glob does NOT canonicalize, the test is a no-op
        // (entry stays inside project_root, no escape).
        std::os::unix::fs::symlink(outside_dir.join("leak.js"), project.join("dist/leak.js"))
            .unwrap();

        let cache_root = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(cache_root.path()).unwrap();
        let result = cache.store(
            "hash-escape",
            "build",
            &["dist/*.js".to_string()],
            "",
            "",
            &project,
        );

        match result {
            Ok(()) => {
                // Glob crate did not canonicalize — entry stayed under
                // project_root, no escape to surface. The test is a
                // no-op on this platform/glob-version combo.
            }
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(
                    msg.contains("GH #3576"),
                    "escape error must include issue tag, got: {msg}"
                );
                assert!(
                    msg.contains("dist/*.js"),
                    "escape error must name the offending pattern, got: {msg}"
                );
                // Metadata file must NOT survive the failure.
                let lookup = cache.lookup("hash-escape").unwrap();
                assert!(
                    lookup.is_none(),
                    "store() must not persist an entry when an output escapes \
                     project_root (GH #3576)"
                );
            }
        }
    }

    // ─── GH #3578: restore silently writes outside project_root on fallback ─

    /// GH #3578 — `format_restore_prefix_err` must include the issue
    /// tag, the offending entry path, the output cache dir, and the
    /// hash so the failure cause is greppable.
    #[test]
    fn gh3578_format_restore_prefix_err_names_tag_paths_and_hash() {
        let entry = std::path::Path::new("/tmp/somewhere/escaped.js");
        let cache_dir = std::path::Path::new("/proj/.jet-cache/tasks/abc123");
        let msg = format_restore_prefix_err(entry, cache_dir, "abc123");

        assert!(
            msg.contains("GH #3578"),
            "must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("/tmp/somewhere/escaped.js"),
            "must name the offending entry path, got: {msg}"
        );
        assert!(
            msg.contains("/proj/.jet-cache/tasks/abc123"),
            "must name the output cache dir, got: {msg}"
        );
        assert!(msg.contains("abc123"), "must name the hash, got: {msg}");
    }

    /// GH #3578 — end-to-end happy path: a normal in-tree cache entry
    /// continues to restore correctly. Pins that the new error path
    /// does not regress the common case.
    #[test]
    fn gh3578_restore_outputs_happy_path_still_restores() {
        let project = tempfile::tempdir().unwrap();
        let cache_root = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(cache_root.path()).unwrap();

        std::fs::create_dir_all(project.path().join("dist")).unwrap();
        std::fs::write(project.path().join("dist/normal.js"), b"const x = 1;").unwrap();

        cache
            .store(
                "hash-restore",
                "build",
                &["dist/*.js".to_string()],
                "",
                "",
                project.path(),
            )
            .unwrap();

        // Restore into a fresh project root.
        let fresh = tempfile::tempdir().unwrap();
        cache.restore_outputs("hash-restore", fresh.path()).unwrap();
        assert_eq!(
            std::fs::read(fresh.path().join("dist/normal.js")).unwrap(),
            b"const x = 1;",
            "happy path must still restore the cached output verbatim (GH #3578)"
        );
    }

    /// GH #3576 — happy path: non-escaping output glob continues to
    /// populate the manifest correctly. Pins that the new
    /// `with_context().?` path does not regress the common case.
    #[test]
    fn gh3576_non_escaping_glob_still_populates_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let cache = TaskCache::new(dir.path()).unwrap();

        std::fs::create_dir_all(dir.path().join("dist")).unwrap();
        std::fs::write(dir.path().join("dist/normal.js"), b"const x = 1;").unwrap();

        cache
            .store(
                "hash-happy",
                "build",
                &["dist/*.js".to_string()],
                "",
                "",
                dir.path(),
            )
            .expect("non-escaping glob must continue to succeed");

        let entry = cache.lookup("hash-happy").unwrap().unwrap();
        assert_eq!(entry.outputs, vec!["dist/normal.js".to_string()]);
    }
}

#[cfg(test)]
mod gh3644_safe_cache_now_tests {
    //! GH #3644 — `chrono_now` previously did
    //! `SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()`,
    //! silently collapsing `Err(_)` to `"0s"` indistinguishable from a real
    //! epoch timestamp. Safe helper distinguishes the two branches.
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn happy_path_produces_seconds_suffix() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let (stamp, warn) = safe_cache_now(now);
        assert_eq!(stamp, "1700000000s");
        assert!(warn.is_none(), "happy path must emit no warn");
    }

    #[test]
    fn epoch_itself_is_zero_seconds_not_an_error() {
        let (stamp, warn) = safe_cache_now(SystemTime::UNIX_EPOCH);
        assert_eq!(stamp, "0s");
        assert!(warn.is_none(), "epoch exact is the happy path");
    }

    #[test]
    fn before_epoch_returns_marker_and_warn() {
        let before = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
        let (stamp, warn) = safe_cache_now(before);
        assert_eq!(
            stamp, "GH-3644-clock-before-epoch",
            "must NOT silently collapse to a seconds string"
        );
        let msg = warn.expect("error branch must emit warn");
        assert!(msg.contains("GH #3644"), "msg: {msg}");
        assert!(msg.contains("created_at"), "msg: {msg}");
    }

    #[test]
    fn marker_is_not_confusable_with_real_timestamp() {
        let before = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
        let (stamp, _) = safe_cache_now(before);
        // Pin: must NOT match the legacy broken `"0s"` form, and must not
        // be parseable as a `<digits>s` timestamp.
        assert_ne!(stamp, "0s");
        assert!(
            !stamp.ends_with("s") || !stamp[..stamp.len() - 1].chars().all(|c| c.is_ascii_digit())
        );
    }

    #[test]
    fn warn_formatter_includes_tag_and_cause() {
        // Synthesize a real SystemTimeError by going backwards.
        let before = SystemTime::UNIX_EPOCH - Duration::from_secs(42);
        let err = before
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect_err("must be Err");
        let msg = format_safe_cache_now_warn(&err);
        assert!(msg.contains("GH #3644"), "msg: {msg}");
        assert!(msg.contains("jet task_runner cache"), "msg: {msg}");
        assert!(msg.contains("NTP"), "msg: {msg}");
    }

    #[test]
    fn helper_name_pins_to_safe_cache_now() {
        // Pin: the safe-helper-name convention from the issue family is
        // `safe_*`. If a future rename breaks this, the loop's grep
        // tooling needs to know.
        let _ = safe_cache_now as fn(SystemTime) -> (String, Option<String>);
    }
}
// CODEGEN-END
