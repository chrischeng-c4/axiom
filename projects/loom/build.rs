//! Build script: stamp `LOOM_GIT_SHA`, `LOOM_BUILT_AT`, and `LOOM_TARGET` into
//! the binary so the standard CLI ops (`upgrade` picks the matching release
//! asset; `report-issue` reports provenance) work without a server (#475).
//!
//! All three are best-effort: outside a git checkout (e.g. a source tarball)
//! the sha falls back to "unknown". Nothing here fails the build.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Re-run when HEAD moves so the stamped sha stays current. The workspace
    // `.git` lives 2 levels up from projects/loom/; in a linked worktree `.git`
    // is a file rather than a dir, so guard the rerun hint.
    if std::path::Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=LOOM_GIT_SHA={git_sha}");

    // Seconds since the epoch — unambiguous, no date crate.
    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LOOM_BUILT_AT={built_at}");

    // The exact target triple cargo built for, so `loom upgrade` can select the
    // matching `loom-<target>.tar.gz` asset. Cargo always sets `TARGET` here.
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LOOM_TARGET={target}");
}

/// Best-effort short SHA of HEAD. Returns `None` outside a git workspace.
fn short_sha() -> Option<String> {
    let out = Command::new("git").args(["rev-parse", "--short=8", "HEAD"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}
