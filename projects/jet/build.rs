// Stamp build provenance into the binary for the standard CLI ops
// (`llm` / `upgrade` / `issue`, via `cli-std`): the target triple
// `jet upgrade` matches against release assets, plus a best-effort git sha and
// build timestamp. All three are best-effort and never fail the build.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Re-run when HEAD moves so the stamped sha stays current. In a linked
    // worktree `../../.git` is a file, not a dir, so this guard simply skips
    // the hint there — the sha is still resolved by `git` below.
    if std::path::Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=JET_GIT_SHA={git_sha}");

    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=JET_BUILT_AT={built_at}");

    // Cargo always sets TARGET for build scripts — the exact triple this binary
    // is built for, e.g. `aarch64-apple-darwin`.
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=JET_TARGET={target}");
}

/// Best-effort short SHA of HEAD; `None` outside a git checkout.
fn short_sha() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!sha.is_empty()).then_some(sha)
}
