// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! Build script: stamp provenance (`VAT_TARGET` / `VAT_GIT_SHA` /
//! `VAT_BUILT_AT`) so `vat upgrade` can pick the matching release asset and
//! `vat report-issue` can attach build diagnostics; then compile the vendored
//! google.pubsub.v1 proto for the built-in Pub/Sub emulator. The proto step is a
//! no-op for a lean build (no `emulator` feature) or before the proto is
//! vendored, so the build never depends on a system protoc.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    stamp_provenance();
    compile_pubsub_proto();
}

/// Stamp build provenance into the binary. All three are best-effort: outside a
/// git checkout the sha falls back to "unknown"; `TARGET` is always set by cargo
/// for build scripts.
fn stamp_provenance() {
    // Re-run when HEAD moves so the stamped sha stays current. The workspace
    // `.git` lives 2 levels up from projects/vat/; in a linked worktree `.git`
    // is a file rather than a dir, so guard the rerun hint.
    if std::path::Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=VAT_GIT_SHA={git_sha}");

    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=VAT_BUILT_AT={built_at}");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=VAT_TARGET={target}");
}

/// Best-effort short SHA of HEAD. Returns `None` outside a git workspace.
fn short_sha() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
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

fn compile_pubsub_proto() {
    // Cargo exposes enabled features as CARGO_FEATURE_<NAME>.
    if std::env::var_os("CARGO_FEATURE_EMULATOR").is_none() {
        return;
    }
    let proto = "proto/google/pubsub/v1/pubsub.proto";
    if !std::path::Path::new(proto).exists() {
        return;
    }
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc binary");
    std::env::set_var("PROTOC", protoc);
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&[proto], &["proto"])
        .expect("compile google.pubsub.v1 proto");
    println!("cargo:rerun-if-changed={proto}");
}
// CODEGEN-END
