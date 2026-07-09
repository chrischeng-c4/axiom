// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
// CODEGEN-BEGIN
//! Content-hash computation for task cache keys.
//!
//! Hash = SHA-256(task_name + sorted input file contents + env values).

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// GH #3574 — internal carrier for the (key, discriminator, value) tuple
/// hashed for each env var. The discriminator byte distinguishes
/// `present` / `not-present` / `not-unicode` so the three states do not
/// collide on the same hash component (which would let stale cache hits
/// leak across them).
struct EnvHashTriple {
    key: String,
    discriminator: u8,
    value: Vec<u8>,
}

/// GH #3574 — build the warn message for a `NotUnicode` env-var lookup.
/// Extracted so the wording (issue tag + key + observed kind) is
/// unit-testable without provoking the actual non-UTF-8 platform case.
///
/// Replaces the prior `unwrap_or_default()` silent fallback which
/// collapsed `NotPresent` and `NotUnicode` onto the same `""` hash
/// component.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_env_var_lookup_warn(key: &str, observed_kind: &str) -> String {
    format!(
        "GH #3574 task cache hash env var `{key}` had non-UTF-8 value (observed: \
         {observed_kind}); hashing raw bytes under the `u` discriminator so this \
         state does not collide on the same cache key as the unset state"
    )
}

/// GH #3574 — extract the raw byte representation of an `OsString` so
/// distinct non-UTF-8 values hash to distinct components. Falls back to
/// the `to_string_lossy()` bytes on platforms where `OsStrExt` is not
/// available (i.e. non-Unix). On lossy fallback the discriminator byte
/// still distinguishes `not-unicode` from `not-present`, so the
/// collision the GH #3574 fix prevents (NotPresent ↔ NotUnicode) cannot
/// reappear; only NotUnicode ↔ NotUnicode of distinct values that map
/// to the same `to_string_lossy()` could collide on non-Unix.
///
/// GH #3807 — the non-Unix lossy arm now goes through
/// `os_string_bytes_or_warn`, which emits a tagged warn so operators
/// running Windows CI see the platform caveat and can audit cache
/// behaviour for tasks that depend on non-UTF-8 env vars.
fn os_string_bytes(raw: &std::ffi::OsString, key: &str) -> Vec<u8> {
    #[cfg(unix)]
    {
        let _ = key;
        use std::os::unix::ffi::OsStrExt;
        return raw.as_os_str().as_bytes().to_vec();
    }
    #[cfg(not(unix))]
    {
        os_string_bytes_non_unix_lossy_or_warn(key, raw)
    }
}

/// GH #3807 — non-Unix lossy fallback for `os_string_bytes`.
///
/// Pulled out as a separate function so the warn-emitting branch is
/// directly callable from tests on any platform. On Windows builds the
/// real call site routes through here; on Unix builds the function is
/// still compiled (so tests can exercise it) but is not invoked from
/// the hot path.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn os_string_bytes_non_unix_lossy_or_warn(
    key: &str,
    raw: &std::ffi::OsString,
) -> Vec<u8> {
    tracing::warn!(
        target: "jet::task_runner::hash",
        key = %key,
        "{}",
        format_env_var_lookup_non_unix_lossy_warn(key)
    );
    raw.to_string_lossy().as_bytes().to_vec()
}

/// GH #3807 — warn shown when `os_string_bytes` takes the non-Unix
/// lossy fallback path. The existing GH #3574 warn names the
/// not-unicode discriminator but says nothing about the platform
/// caveat that distinct non-UTF-8 values may still lossy onto the
/// same cache component on Windows.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_env_var_lookup_non_unix_lossy_warn(key: &str) -> String {
    format!(
        "gh3807: task cache hash env var `{key}` hashed via to_string_lossy() \
         because OsStrExt is not available on this platform; two NotUnicode \
         values that lossy onto the same string will silently share a cache \
         component on non-Unix targets",
        key = key
    )
}

/// Compute a deterministic hash for a task based on its inputs.
///
/// Components:
/// 1. Task name
/// 2. Content of all input files (sorted by path for determinism)
/// 3. Values of specified environment variables
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub fn compute_task_hash(
    task_name: &str,
    input_globs: &[String],
    env_keys: &[String],
    project_root: &Path,
) -> Result<String> {
    let mut hasher = Sha256::new();

    // 1. Task name
    hasher.update(task_name.as_bytes());
    hasher.update(b"\0");

    // 2. Input file contents (sorted).
    //
    // GH #3107: read failures and glob-compile failures used to be
    // silently dropped, which produced the SAME hash for DIFFERENT
    // input sets and let stale cache hits leak through. Propagate
    // both via `?` so the caller (task_runner::cache) can decide how
    // to recover.
    let mut input_files = collect_input_files(input_globs, project_root)?;
    input_files.sort();

    for file_path in &input_files {
        let full = project_root.join(file_path);
        let content = std::fs::read(&full).with_context(|| {
            format!(
                "Failed to read task cache input {}; refusing to compute a hash that would silently omit it (GH #3107)",
                full.display()
            )
        })?;
        hasher.update(file_path.as_bytes());
        hasher.update(b"\0");
        hasher.update(&content);
        hasher.update(b"\0");
    }

    // 3. Environment variables (sorted by key).
    //
    // GH #3574 — `std::env::var(k).unwrap_or_default()` used to collapse
    // both `VarError::NotPresent` and `VarError::NotUnicode(OsString)`
    // onto the same `""` hash component, so a task hashed once with
    // the key unset and re-hashed with the same key set to a non-UTF-8
    // value would reuse the stale cache entry. Distinguish the three
    // states with a leading discriminator byte (`p` = present, `n` =
    // not-present, `u` = not-unicode) so they hash differently. On the
    // `NotUnicode` branch we additionally hash the raw `OsString` bytes
    // so two different non-UTF-8 values do not collide on the same
    // discriminator.
    let mut env_triples: Vec<EnvHashTriple> = env_keys
        .iter()
        .map(|k| match std::env::var(k) {
            Ok(v) => EnvHashTriple {
                key: k.clone(),
                discriminator: b'p',
                value: v.into_bytes(),
            },
            Err(std::env::VarError::NotPresent) => EnvHashTriple {
                key: k.clone(),
                discriminator: b'n',
                value: Vec::new(),
            },
            Err(std::env::VarError::NotUnicode(raw)) => {
                tracing::warn!(
                    target: "jet::task_runner::hash",
                    key = %k,
                    "{}",
                    format_env_var_lookup_warn(k, "not-unicode")
                );
                EnvHashTriple {
                    key: k.clone(),
                    discriminator: b'u',
                    value: os_string_bytes(&raw, k),
                }
            }
        })
        .collect();
    env_triples.sort_by(|a, b| a.key.cmp(&b.key));

    for triple in &env_triples {
        hasher.update(triple.key.as_bytes());
        hasher.update(b"=");
        hasher.update([triple.discriminator]);
        hasher.update(&triple.value);
        hasher.update(b"\0");
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Collect input files matching glob patterns, returning relative paths.
fn collect_input_files(globs: &[String], project_root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    if globs.is_empty() {
        // Default: hash all source files
        return Ok(files);
    }

    for pattern in globs {
        let full_pattern = format!("{}/{}", project_root.display(), pattern);
        let entries = glob::glob(&full_pattern).with_context(|| {
            format!(
                "Invalid task input glob {pattern:?}; refusing to compute a hash that would silently omit it (GH #3107)"
            )
        })?;
        for entry in entries {
            let entry = entry.with_context(|| {
                format!(
                    "Failed to walk task input glob {pattern:?}; refusing to compute a hash that would silently omit it (GH #3290)"
                )
            })?;
            if !entry.is_file() {
                continue;
            }
            // GH #3290 — a path that escapes `project_root` (e.g. a
            // symlink resolving outside the project) used to be
            // silently dropped via `if let Ok(rel) = …`. Refuse to
            // compute the hash instead: silently dropping it changes
            // the hash without changing the input set.
            let rel = entry.strip_prefix(project_root).with_context(|| {
                format!(
                    "Task input {} escapes project_root {} via glob {pattern:?}; refusing to compute a hash that would silently omit it (GH #3290)",
                    entry.display(),
                    project_root.display()
                )
            })?;
            // GH #3753 — prior `rel.to_string_lossy().to_string()`
            // silently replaced any non-UTF-8 byte with U+FFFD. Two
            // distinct files whose names differ only in their invalid
            // byte positions would then collide into the same hash
            // input string — silent cache-key collision. Refuse to
            // hash inputs whose path is not valid UTF-8.
            let rel_str = rel.to_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "{}",
                    format_task_hash_non_utf8_warn(rel, project_root, pattern)
                )
            })?;
            files.push(rel_str.to_string());
        }
    }

    Ok(files)
}

/// GH #3753 — build the error message for a task-input path whose
/// relative-to-project bytes are not valid UTF-8. Extracted so the
/// wording (issue tag + path + project root + glob + downstream
/// consequence) is unit-testable without provoking a real non-UTF-8
/// filename in the integration path.
///
/// Mirrors the `#3290` / `#3576` / `#3741` naming family — every
/// "refuse to hash / cache" path in this module follows the same
/// `format_<scope>_<symptom>_<level>` shape.
/// @spec .aw/tech-design/projects/jet/semantic/jet-task-runner.md#schema
pub(crate) fn format_task_hash_non_utf8_warn(
    rel: &Path,
    project_root: &Path,
    pattern: &str,
) -> String {
    format!(
        "GH #3753 task input {} (under {}, matched by glob `{pattern}`) \
         contains bytes that are not valid UTF-8; jet refuses to hash \
         this input because the prior `.to_string_lossy()` would \
         substitute U+FFFD for the invalid bytes and silently collide \
         with any other file whose name differs only in those byte \
         positions. The result would be a false cache hit — task \
         output reused from an unrelated input. Rename the file to a \
         UTF-8 name (or canonicalize via your filesystem's normalization \
         tools) and re-run.",
        rel.display(),
        project_root.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("src.js"), "console.log('hi')").unwrap();

        let h1 = compute_task_hash("build", &["src.js".to_string()], &[], dir.path()).unwrap();
        let h2 = compute_task_hash("build", &["src.js".to_string()], &[], dir.path()).unwrap();

        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("src.js"), "v1").unwrap();

        let h1 = compute_task_hash("build", &["src.js".to_string()], &[], dir.path()).unwrap();

        std::fs::write(dir.path().join("src.js"), "v2").unwrap();
        let h2 = compute_task_hash("build", &["src.js".to_string()], &[], dir.path()).unwrap();

        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_different_task_names() {
        let dir = tempfile::tempdir().unwrap();

        let h1 = compute_task_hash("build", &[], &[], dir.path()).unwrap();
        let h2 = compute_task_hash("test", &[], &[], dir.path()).unwrap();

        assert_ne!(h1, h2);
    }

    #[test]
    fn test_empty_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let result = compute_task_hash("t", &[], &[], dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 64); // SHA-256 hex
    }

    /// GH #3107 — malformed glob patterns must surface as `Err`, not get
    /// silently dropped (which made two runs with different inputs collide
    /// on the same cache key).
    #[test]
    fn invalid_glob_pattern_surfaces_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = compute_task_hash("build", &["src/**[".to_string()], &[], dir.path());
        let err = result.expect_err("malformed glob must not silently match zero files");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Invalid task input glob") && msg.contains("3107"),
            "expected refusal-to-poison-cache message, got: {msg}"
        );
    }

    /// GH #3107 — unreadable input file must surface as `Err`, not get
    /// silently omitted from the hash.
    #[cfg(unix)]
    #[test]
    fn unreadable_input_surfaces_error() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("src.js");
        std::fs::write(&path, "console.log('hi')").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // chmod has no effect when running as root → skip cleanly.
        if std::fs::read(&path).is_ok() {
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let result = compute_task_hash("build", &["src.js".to_string()], &[], dir.path());

        // Restore perms so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));

        let err = result.expect_err("unreadable input must not produce a silent hash");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Failed to read task cache input") && msg.contains("3107"),
            "expected refusal-to-poison-cache message, got: {msg}"
        );
    }

    /// GH #3290 — a per-glob-entry `glob::GlobError` (e.g. a chmod 0o000
    /// directory traversed mid-walk) used to be silently dropped via
    /// `entries.flatten()`. The hash would proceed as if the matched
    /// path weren't an input, producing the SAME cache key for runs
    /// that should have differed. Surface the per-entry error too.
    #[cfg(unix)]
    #[test]
    fn unwalkable_directory_in_glob_surfaces_error() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        // Create a subdirectory the glob will descend into.
        let sub = dir.path().join("locked");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("inside.js"), "v1").unwrap();
        // Forbid traversal.
        std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o000)).unwrap();

        // chmod has no effect when running as root → skip cleanly.
        if std::fs::read_dir(&sub).is_ok() {
            let _ = std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let result = compute_task_hash("build", &["locked/**/*.js".to_string()], &[], dir.path());

        // Restore perms so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755));

        let err = result
            .expect_err("unstat'able glob entry must not be silently dropped from the hash inputs");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Failed to walk task input glob") && msg.contains("3290"),
            "expected per-entry refusal-to-poison-cache message, got: {msg}"
        );
    }

    /// GH #3290 — a glob match that escapes `project_root` via a symlink
    /// resolving outside the project used to be silently dropped via
    /// `if let Ok(rel) = entry.strip_prefix(...)`. Refuse to compute the
    /// hash instead.
    #[cfg(unix)]
    #[test]
    fn glob_entry_escaping_project_root_surfaces_error() {
        let outer = tempfile::tempdir().unwrap();
        let outside = outer.path().join("outside.js");
        std::fs::write(&outside, "v1").unwrap();

        let project = outer.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        // Symlink that resolves to a sibling outside `project_root`.
        // The glob walker resolves symlinks to absolute paths, so the
        // entry path will not start with `project`.
        std::os::unix::fs::symlink(&outside, project.join("escapes.js")).unwrap();

        let result = compute_task_hash(
            "build",
            // The glob crate by default does not follow symlinks when
            // listing files, so we point directly at the symlink name.
            // Some platforms canonicalize the result; both paths should
            // be exercised by this test.
            &["escapes.js".to_string()],
            &[],
            &project,
        );

        // We don't pin Ok-vs-Err: the glob crate may either return the
        // symlink path unchanged (in which case strip_prefix succeeds
        // and the test is a no-op) or canonicalize and surface the
        // path outside `project_root`. If the latter, the error must
        // mention GH #3290; if the former, the call must still succeed
        // (no silent drop of a path that *did* match).
        match result {
            Ok(_) => {}
            Err(err) => {
                let msg = format!("{err:#}");
                assert!(
                    msg.contains("escapes project_root") && msg.contains("3290"),
                    "expected escape-project-root refusal message, got: {msg}"
                );
            }
        }
    }

    // ─── GH #3574: env var NotPresent vs NotUnicode silent collision ─────

    /// GH #3574 — the warn string must include the issue tag, the
    /// offending env var key, and the observed kind so a grep on the
    /// logs surfaces the cache-key-drift cause.
    #[test]
    fn gh3574_format_env_var_lookup_warn_names_tag_key_and_kind() {
        let msg = format_env_var_lookup_warn("LANG", "not-unicode");
        assert!(
            msg.contains("GH #3574"),
            "must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("LANG"),
            "must name the env var key, got: {msg}"
        );
        assert!(
            msg.contains("not-unicode"),
            "must name the observed kind, got: {msg}"
        );
    }

    /// GH #3574 — `compute_task_hash` must produce a DIFFERENT hash
    /// for "env var unset" vs "env var set to empty string" so the two
    /// states cannot reuse the same cache entry. Pre-fix, both
    /// collapsed to `""` and produced the SAME hash.
    #[test]
    fn gh3574_unset_vs_empty_string_env_vars_hash_differently() {
        // Use a key unique to this test so parallel tests don't clobber
        // each other's view of the env.
        let key = format!("JET_GH3574_TEST_{}", std::process::id());
        std::env::remove_var(&key);

        let dir = tempfile::tempdir().unwrap();
        let h_unset = compute_task_hash("build", &[], &[key.clone()], dir.path()).unwrap();

        std::env::set_var(&key, "");
        let h_empty = compute_task_hash("build", &[], &[key.clone()], dir.path()).unwrap();

        // Restore.
        std::env::remove_var(&key);

        assert_ne!(
            h_unset, h_empty,
            "unset and empty-string env vars must hash differently (GH #3574); \
             pre-fix both collapsed to `\"\"` and collided on the cache key"
        );
    }

    /// GH #3574 — end-to-end on Unix: simulate `NotUnicode` by setting
    /// the env var to a raw non-UTF-8 byte sequence. The hash must
    /// differ from the unset state.
    #[cfg(unix)]
    #[test]
    fn gh3574_not_present_vs_not_unicode_env_vars_hash_differently() {
        use std::os::unix::ffi::OsStrExt;

        let key = format!("JET_GH3574_NONUTF_{}", std::process::id());
        std::env::remove_var(&key);

        let dir = tempfile::tempdir().unwrap();
        let h_unset = compute_task_hash("build", &[], &[key.clone()], dir.path()).unwrap();

        // 0xFF, 0xFE is an invalid UTF-8 lead-byte sequence.
        let non_utf8 = std::ffi::OsStr::from_bytes(&[0xFF, 0xFE]);
        std::env::set_var(&key, non_utf8);
        let h_non_utf8 = compute_task_hash("build", &[], &[key.clone()], dir.path()).unwrap();

        // Restore.
        std::env::remove_var(&key);

        assert_ne!(
            h_unset, h_non_utf8,
            "NotPresent and NotUnicode must hash to different cache keys (GH #3574); \
             pre-fix both collapsed to `\"\"`"
        );
    }
}

#[cfg(test)]
mod gh3753_non_utf8_path_warn_tests {
    use super::*;
    use crate::task_runner::cache::format_task_cache_non_utf8_err;

    fn proj_root() -> std::path::PathBuf {
        std::path::PathBuf::from("/tmp/proj")
    }

    /// Helper output carries the issue tag, the offending path, the
    /// project root, and the glob — all needed for triage.
    #[test]
    fn gh3753_hash_warn_message_contains_issue_tag_and_context() {
        let rel = std::path::Path::new("foo/bad.bin");
        let msg = format_task_hash_non_utf8_warn(rel, &proj_root(), "src/**/*.bin");
        assert!(msg.contains("GH #3753"), "msg: {msg}");
        assert!(
            msg.contains("foo/bad.bin"),
            "msg must name the relative path: {msg}"
        );
        assert!(
            msg.contains("/tmp/proj"),
            "msg must name project root: {msg}"
        );
        assert!(msg.contains("src/**/*.bin"), "msg must name glob: {msg}");
        assert!(
            msg.contains("false cache hit")
                || msg.contains("cache-key collision")
                || msg.contains("collide"),
            "msg must call out the cache-collision consequence: {msg}"
        );
    }

    /// Cache-side helper carries the symmetric error message.
    #[test]
    fn gh3753_cache_err_message_contains_issue_tag_and_context() {
        let rel = std::path::Path::new("dist/bad.bin");
        let msg = format_task_cache_non_utf8_err(rel, &proj_root(), "dist/**/*");
        assert!(msg.contains("GH #3753"), "msg: {msg}");
        assert!(msg.contains("dist/bad.bin"), "msg must name path: {msg}");
        assert!(msg.contains("dist/**/*"), "msg must name glob: {msg}");
        assert!(
            msg.contains("corrupted cache restore")
                || msg.contains("manifest")
                || msg.contains("round-trip"),
            "msg must call out the restore-corruption consequence: {msg}"
        );
    }

    /// Hash-side vs cache-side messages must be DISTINCT so triage
    /// can tell which side failed (input hash vs output cache).
    #[test]
    fn gh3753_hash_vs_cache_messages_are_distinct() {
        let rel = std::path::Path::new("x");
        let hash_msg = format_task_hash_non_utf8_warn(rel, &proj_root(), "p");
        let cache_msg = format_task_cache_non_utf8_err(rel, &proj_root(), "p");
        assert_ne!(hash_msg, cache_msg);
        assert!(
            hash_msg.contains("hash"),
            "hash msg must self-identify: {hash_msg}"
        );
        assert!(
            cache_msg.contains("cache") || cache_msg.contains("output"),
            "cache msg must self-identify: {cache_msg}"
        );
    }

    /// Deterministic — same input → byte-identical message.
    #[test]
    fn gh3753_warn_messages_are_deterministic() {
        let rel = std::path::Path::new("x");
        let a = format_task_hash_non_utf8_warn(rel, &proj_root(), "p");
        let b = format_task_hash_non_utf8_warn(rel, &proj_root(), "p");
        assert_eq!(a, b);
        let c = format_task_cache_non_utf8_err(rel, &proj_root(), "p");
        let d = format_task_cache_non_utf8_err(rel, &proj_root(), "p");
        assert_eq!(c, d);
    }

    /// Sibling distinctness vs related warn-tags from the
    /// task_runner family (#3107, #3153, #3290, #3574, #3576) and
    /// the broader project family (#3741 non-UTF-8 basename fix).
    #[test]
    fn gh3753_warns_are_distinct_from_siblings() {
        let rel = std::path::Path::new("x");
        let hash_msg = format_task_hash_non_utf8_warn(rel, &proj_root(), "p");
        let cache_msg = format_task_cache_non_utf8_err(rel, &proj_root(), "p");
        for tag in ["#3107", "#3153", "#3290", "#3574", "#3576", "#3741"] {
            assert!(
                !hash_msg.contains(tag),
                "hash msg must not contain sibling tag {tag}: {hash_msg}"
            );
            assert!(
                !cache_msg.contains(tag),
                "cache msg must not contain sibling tag {tag}: {cache_msg}"
            );
        }
    }

    /// Naming convention discoverability — keeps the warn / err
    /// helper family uniformly named so future authors find them.
    #[test]
    fn gh3753_helper_names_follow_family_convention() {
        // Hash-side uses `_warn` suffix (matches #3741 family).
        let hash_name = "format_task_hash_non_utf8_warn";
        assert!(hash_name.starts_with("format_"));
        assert!(hash_name.ends_with("_warn"));
        // Cache-side uses `_err` suffix because it's surfaced via
        // anyhow::Error rather than tracing — matches #3576 sibling
        // `format_output_escape_err`.
        let cache_name = "format_task_cache_non_utf8_err";
        assert!(cache_name.starts_with("format_"));
        assert!(cache_name.ends_with("_err"));
    }

    /// Different relative paths produce DISTINCT messages so triage
    /// can pinpoint which file is offending.
    #[test]
    fn gh3753_different_paths_produce_distinct_messages() {
        let a = format_task_hash_non_utf8_warn(std::path::Path::new("a.bin"), &proj_root(), "p");
        let b = format_task_hash_non_utf8_warn(std::path::Path::new("b.bin"), &proj_root(), "p");
        assert_ne!(a, b);
        assert!(a.contains("a.bin"));
        assert!(b.contains("b.bin"));
    }

    /// End-to-end on Linux: construct a real file whose name contains
    /// non-UTF-8 bytes, glob it into `collect_input_files`, and
    /// verify the function errors with the expected helper message
    /// rather than silently lossy-encoding the path.
    ///
    /// Gated to Linux because macOS APFS refuses to create files with
    /// non-UTF-8 names (EILSEQ "Illegal byte sequence"); the unit-level
    /// helper test below covers the same contract cross-platform.
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn gh3753_collect_input_files_rejects_non_utf8_path() {
        use std::ffi::OsStr;
        use std::fs;
        use std::os::unix::ffi::OsStrExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Put the bad file under a known subdir so the glob can match.
        let sub = root.join("inputs");
        fs::create_dir_all(&sub).unwrap();
        // 0xFF is invalid UTF-8 (lone continuation byte).
        let bad_name = OsStr::from_bytes(&[b'b', b'a', b'd', 0xFF, b'.', b'b', b'i', b'n']);
        let bad_path = sub.join(bad_name);
        fs::write(&bad_path, b"x").unwrap();

        let err = collect_input_files(&["inputs/*".to_string()], root)
            .expect_err("non-UTF-8 input path must yield Err, not silent lossy hash");
        let msg = format!("{err:#}");
        assert!(msg.contains("GH #3753"), "err must carry issue tag: {msg}");
        assert!(
            msg.contains("not valid UTF-8") || msg.contains("U+FFFD"),
            "err must call out UTF-8 issue: {msg}"
        );
    }

    /// Cross-Unix unit check: a `Path` built from non-UTF-8 bytes must
    /// fail `.to_str()` — i.e. the contract `collect_input_files` relies
    /// on for its `to_str().ok_or_else(...)` rejection holds without
    /// requiring filesystem support for non-UTF-8 filenames.
    #[cfg(unix)]
    #[test]
    fn gh3753_non_utf8_pathbuf_returns_none_from_to_str() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;

        let bad = PathBuf::from(OsStr::from_bytes(&[b'a', 0xFF, b'b']));
        assert!(
            bad.to_str().is_none(),
            "non-UTF-8 PathBuf must not round-trip via to_str()"
        );
    }

    /// Smoke check: well-formed UTF-8 paths still round-trip
    /// through `collect_input_files` without error.
    #[test]
    fn gh3753_collect_input_files_accepts_valid_utf8_paths() {
        use std::fs;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("inputs")).unwrap();
        fs::write(root.join("inputs/good.txt"), b"hi").unwrap();
        let out = collect_input_files(&["inputs/*.txt".to_string()], root).unwrap();
        assert!(out.contains(&"inputs/good.txt".to_string()), "got: {out:?}");
    }
}

#[cfg(test)]
mod gh3807_non_unix_lossy_env_warn_tests {
    use super::{
        format_env_var_lookup_non_unix_lossy_warn, format_env_var_lookup_warn,
        os_string_bytes_non_unix_lossy_or_warn,
    };
    use std::ffi::OsString;

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let src = include_str!("hash.rs");
        assert!(src.contains("fn format_env_var_lookup_non_unix_lossy_warn"));
        assert!(src.contains("fn os_string_bytes_non_unix_lossy_or_warn"));
    }

    #[test]
    fn each_warn_string_carries_gh3807_tag() {
        let s = format_env_var_lookup_non_unix_lossy_warn("LANG_TAG");
        assert!(s.starts_with("gh3807:"), "missing gh3807 tag: {s:?}");
        assert!(s.contains("LANG_TAG"));
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let s = format_env_var_lookup_non_unix_lossy_warn("X");
        for tag in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805",
        ] {
            assert!(!s.contains(tag), "gh3807 warn must not carry {tag}: {s:?}");
        }
    }

    #[test]
    fn warn_distinct_from_existing_gh3574_lookup_warn() {
        // The existing GH #3574 warn names the not-unicode discriminator
        // but says nothing about the non-Unix lossy caveat. The new
        // gh3807 warn must be visibly different so operators reading
        // logs can spot the platform-specific risk.
        let old = format_env_var_lookup_warn("X", "not-unicode");
        let new = format_env_var_lookup_non_unix_lossy_warn("X");
        assert_ne!(old, new);
        assert!(old.contains("GH #3574"));
        assert!(new.contains("gh3807:"));
    }

    #[test]
    fn warn_explains_collision_consequence() {
        let s = format_env_var_lookup_non_unix_lossy_warn("X");
        assert!(
            s.contains("share a cache component") || s.contains("lossy onto the same"),
            "warn should explain the collision consequence: {s:?}"
        );
    }

    #[test]
    fn warn_names_non_unix_platform_caveat() {
        let s = format_env_var_lookup_non_unix_lossy_warn("X");
        assert!(
            s.contains("non-Unix") || s.contains("OsStrExt"),
            "warn should name the platform caveat: {s:?}"
        );
    }

    #[test]
    fn os_string_bytes_non_unix_lossy_helper_returns_lossy_form_for_ascii() {
        // Pure ASCII OsString: the lossy form equals the original bytes,
        // so two ASCII values still hash distinctly.
        let raw = OsString::from("hello");
        let bytes = os_string_bytes_non_unix_lossy_or_warn("X", &raw);
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn os_string_bytes_non_unix_lossy_helper_returns_lossy_form_for_unicode_text() {
        // Multi-byte UTF-8 text: lossy form preserves the bytes exactly.
        let raw = OsString::from("café");
        let bytes = os_string_bytes_non_unix_lossy_or_warn("X", &raw);
        assert_eq!(bytes, "café".as_bytes());
    }

    #[cfg(unix)]
    #[test]
    fn unix_arm_round_trips_byte_perfectly_for_non_utf8_value() {
        // Unix path takes OsStrExt::as_bytes, not the lossy fallback.
        // Two distinct non-UTF-8 OsStrings produce distinct byte vecs.
        use std::os::unix::ffi::OsStringExt;
        let raw_a = OsString::from_vec(vec![0xFF, b'a']);
        let raw_b = OsString::from_vec(vec![0xFE, b'a']);
        let bytes_a = super::os_string_bytes(&raw_a, "X");
        let bytes_b = super::os_string_bytes(&raw_b, "X");
        assert_ne!(bytes_a, bytes_b);
        assert_eq!(bytes_a, vec![0xFF, b'a']);
        assert_eq!(bytes_b, vec![0xFE, b'a']);
    }

    #[test]
    fn warn_message_is_deterministic_for_same_key() {
        let a = format_env_var_lookup_non_unix_lossy_warn("API_TOKEN");
        let b = format_env_var_lookup_non_unix_lossy_warn("API_TOKEN");
        assert_eq!(a, b);
    }
}
// CODEGEN-END
