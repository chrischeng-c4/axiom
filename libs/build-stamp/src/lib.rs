// CODEGEN-BEGIN
//! Shared `build.rs` stamping: immutable or local git sha, built-at epoch, and target
//! triple as `cargo:rustc-env=<PREFIX>_*` directives.
//!
//! Consuming crates call [`stamp`] with their env-var prefix (e.g. `"LUMEN"`)
//! from their `build.rs`'s `fn main()`. An exact `<PREFIX>_SOURCE_REVISION`
//! carries a full Git SHA into archive builds. Without it, every stamp remains
//! best-effort: outside a git checkout the sha falls back to `"unknown"`, and
//! downstream `env!`/`option_env!` consumers degrade the same way.
//! Nothing here fails the build.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Emit `cargo:rustc-env=<PREFIX>_GIT_SHA=...`, `<PREFIX>_BUILT_AT=...`, and
/// `<PREFIX>_TARGET=...`, plus the `../../.git/HEAD` rerun-if-changed hint.
///
/// `prefix` is the caller's env-var prefix, e.g. `"LUMEN"` for
/// `LUMEN_GIT_SHA` / `LUMEN_BUILT_AT` / `LUMEN_TARGET`.
pub fn stamp(prefix: &str) {
    // Re-run when HEAD moves so the stamped sha stays current. The workspace
    // `.git` lives 2 levels up from the calling crate's build.rs (e.g.
    // `projects/<name>/`); in a linked worktree `.git` is a file rather than
    // a dir, so guard the rerun hint.
    if let Some(hint) = git_head_rerun_hint(Path::new("../../.git/HEAD")) {
        println!("{hint}");
    }

    let source_revision_variable = format!("{prefix}_SOURCE_REVISION");
    println!("cargo:rerun-if-env-changed={source_revision_variable}");
    let git_sha = explicit_source_revision(&source_revision_variable)
        .or_else(short_sha)
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env={prefix}_GIT_SHA={git_sha}");

    let built_at = built_at_now();
    println!("cargo:rustc-env={prefix}_BUILT_AT={built_at}");

    let target = target_from_env();
    println!("cargo:rustc-env={prefix}_TARGET={target}");
}

/// Read a caller-supplied immutable Git revision for archive and image builds.
/// Local builds keep the existing best-effort short-SHA fallback.
fn explicit_source_revision(variable: &str) -> Option<String> {
    decode_source_revision(std::env::var(variable).ok())
}

fn decode_source_revision(value: Option<String>) -> Option<String> {
    let value = value?;
    (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| value.to_ascii_lowercase())
}

/// Cargo's `cargo:rerun-if-changed=<head_path>` directive, only when that
/// path actually exists.
fn git_head_rerun_hint(head_path: &Path) -> Option<String> {
    head_path
        .exists()
        .then(|| format!("cargo:rerun-if-changed={}", head_path.display()))
}

/// Best-effort short SHA of HEAD. Returns `None` outside a git workspace (or
/// when the `git` binary itself is absent).
fn short_sha() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
    decode_short_sha(out.status.success(), &out.stdout)
}

/// Pure decode of a `git rev-parse` invocation's outcome, split out from
/// [`short_sha`] so the fallback path is unit-testable without depending on
/// the environment's git availability.
fn decode_short_sha(success: bool, stdout: &[u8]) -> Option<String> {
    if !success {
        return None;
    }
    let sha = String::from_utf8_lossy(stdout).trim().to_string();
    (!sha.is_empty()).then_some(sha)
}

/// RFC3339-ish UTC timestamp without pulling in a date crate: seconds since
/// the epoch are unambiguous and trivially formattable.
fn built_at_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(format_built_at)
        .unwrap_or_else(|_| "unknown".to_string())
}

fn format_built_at(d: Duration) -> String {
    format!("{}", d.as_secs())
}

/// The exact target triple cargo built for (always set for build scripts),
/// falling back to `"unknown"` if absent.
fn target_from_env() -> String {
    decode_target(std::env::var("TARGET").ok())
}

fn decode_target(v: Option<String>) -> String {
    v.unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_short_sha_none_on_failed_command() {
        assert_eq!(decode_short_sha(false, b"deadbeef\n"), None);
    }

    #[test]
    fn decode_short_sha_none_on_empty_stdout() {
        assert_eq!(decode_short_sha(true, b""), None);
    }

    #[test]
    fn decode_short_sha_trims_trailing_newline() {
        assert_eq!(
            decode_short_sha(true, b"c3ff13cd\n"),
            Some("c3ff13cd".to_string())
        );
    }

    #[test]
    fn explicit_source_revision_accepts_and_normalizes_a_full_git_sha() {
        assert_eq!(
            decode_source_revision(Some("0123456789ABCDEF0123456789ABCDEF01234567".into())),
            Some("0123456789abcdef0123456789abcdef01234567".into())
        );
    }

    #[test]
    fn explicit_source_revision_rejects_short_or_non_hex_values() {
        assert_eq!(decode_source_revision(Some("deadbeef".into())), None);
        assert_eq!(
            decode_source_revision(Some("0123456789abcdef0123456789abcdef0123456z".into())),
            None
        );
        assert_eq!(decode_source_revision(Some(String::new())), None);
    }

    #[test]
    fn short_sha_resolves_inside_this_git_workspace() {
        // Integration-style sanity check: this crate lives inside a git
        // checkout, so the real (non-mocked) code path should resolve.
        assert!(short_sha().is_some());
    }

    #[test]
    fn format_built_at_is_epoch_seconds() {
        assert_eq!(
            format_built_at(Duration::from_secs(1_700_000_000)),
            "1700000000"
        );
    }

    #[test]
    fn decode_target_falls_back_to_unknown_when_absent() {
        assert_eq!(decode_target(None), "unknown");
    }

    #[test]
    fn decode_target_passes_through_when_present() {
        assert_eq!(
            decode_target(Some("aarch64-apple-darwin".to_string())),
            "aarch64-apple-darwin"
        );
    }

    #[test]
    fn git_head_rerun_hint_none_when_path_missing() {
        let missing = Path::new("/nonexistent-path-for-build-stamp-tests/.git/HEAD");
        assert_eq!(git_head_rerun_hint(missing), None);
    }

    #[test]
    fn git_head_rerun_hint_some_when_path_present() {
        let dir = std::env::temp_dir().join(format!("build-stamp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let head = dir.join("HEAD");
        std::fs::write(&head, b"ref: refs/heads/main\n").unwrap();

        let hint = git_head_rerun_hint(&head).unwrap();
        assert_eq!(hint, format!("cargo:rerun-if-changed={}", head.display()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
// CODEGEN-END
