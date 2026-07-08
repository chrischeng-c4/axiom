---
id: libs-build-stamp-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/build-stamp/src/lib.rs`.
capability_refs:
  - id: build-script-version-stamp
    role: primary
    gap: build-script-version-stamp-contract
    claim: build-script-version-stamp-contract
    coverage: full
    rationale: "The source unit implements the build-stamp library capability."
fill_sections: [overview, source, changes]
---

# Standardized libs/build-stamp/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/build-stamp/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `stamp` | libs/build-stamp/src/lib.rs | function | pub | 21 | stamp(prefix: &str) |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Shared `build.rs` stamping: git short-sha, built-at epoch, and target
//! triple as `cargo:rustc-env=<PREFIX>_*` directives.
//!
//! Consuming crates call [`stamp`] with their env-var prefix (e.g. `"LUMEN"`)
//! from their `build.rs`'s `fn main()`. Every stamp is best-effort: outside a
//! git checkout (e.g. a source tarball) the sha falls back to `"unknown"`,
//! and downstream `env!`/`option_env!` consumers degrade the same way.
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

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env={prefix}_GIT_SHA={git_sha}");

    let built_at = built_at_now();
    println!("cargo:rustc-env={prefix}_BUILT_AT={built_at}");

    let target = target_from_env();
    println!("cargo:rustc-env={prefix}_TARGET={target}");
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/build-stamp/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/build-stamp/src/lib.rs` captured during libs codegen standardization.
```
