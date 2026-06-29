// HANDWRITE-BEGIN gap="missing-generator:build-provenance:bf89beec" tracker="pending-tracker" reason="Stamp CAP_TARGET, CAP_GIT_SHA, and CAP_BUILT_AT for cli_std::ToolInfo release asset and diagnostics metadata."
// Stamp build provenance into the binary for the standard CLI ops
// (`llm` / `upgrade` / `issue`, via `cli-std`): the target triple
// lets `cap upgrade` select release assets, and the diagnostics path can
// include a best-effort git sha and build timestamp.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    if std::path::Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=CAP_GIT_SHA={git_sha}");

    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=CAP_BUILT_AT={built_at}");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=CAP_TARGET={target}");
}

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
// HANDWRITE-END
