//! Build script: stamp `MAMBA_GIT_SHA`, `MAMBA_BUILT_AT`, and `MAMBA_TARGET`
//! into the binary for the standard CLI ops (`upgrade` and `report-issue`).
//!
//! Stamps are best-effort. Builds outside a git checkout fall back to
//! "unknown" and never fail because provenance could not be collected.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Always emit at least one narrow cargo:rerun-if-changed directive so Git
    // discovery failure never falls back to whole-package watching.
    println!("cargo:rerun-if-changed=build.rs");

    setup_git_rerun_directives();

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=MAMBA_GIT_SHA={git_sha}");

    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=MAMBA_BUILT_AT={built_at}");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=MAMBA_TARGET={target}");
}

fn setup_git_rerun_directives() {
    let dot_git = Path::new("../../.git");
    if !dot_git.exists() {
        return;
    }

    let git_dir = if dot_git.is_file() {
        println!("cargo:rerun-if-changed={}", dot_git.display());
        let content = match std::fs::read_to_string(dot_git) {
            Ok(c) => c,
            Err(_) => return,
        };
        let line = content.lines().next().unwrap_or("");
        if let Some(rest) = line.strip_prefix("gitdir:") {
            let path_str = rest.trim();
            let p = PathBuf::from(path_str);
            if p.is_relative() {
                dot_git.parent().unwrap_or_else(|| Path::new(".")).join(p)
            } else {
                p
            }
        } else {
            return;
        }
    } else if dot_git.is_dir() {
        dot_git.to_path_buf()
    } else {
        return;
    };

    let commondir_file = git_dir.join("commondir");
    let common_dir = if commondir_file.exists() {
        if let Ok(cd_content) = std::fs::read_to_string(&commondir_file) {
            let cd_str = cd_content.lines().next().unwrap_or("").trim();
            let cd_path = PathBuf::from(cd_str);
            if cd_path.is_relative() {
                git_dir.join(cd_path)
            } else {
                cd_path
            }
        } else {
            git_dir.clone()
        }
    } else {
        git_dir.clone()
    };

    let head_file = git_dir.join("HEAD");
    if head_file.exists() {
        println!("cargo:rerun-if-changed={}", head_file.display());
        if let Ok(head_content) = std::fs::read_to_string(&head_file) {
            let head_line = head_content.lines().next().unwrap_or("").trim();
            if let Some(ref_path_str) = head_line.strip_prefix("ref:") {
                let ref_path_str = ref_path_str.trim();
                let canonical_ref_path = common_dir.join(ref_path_str);
                println!("cargo:rerun-if-changed={}", canonical_ref_path.display());

                let packed_refs = common_dir.join("packed-refs");
                if packed_refs.exists() {
                    println!("cargo:rerun-if-changed={}", packed_refs.display());
                }
            }
        }
    }
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
