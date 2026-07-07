// HANDWRITE-BEGIN gap="missing-generator:unit-test:d757d3fb" tracker="#777" reason="Drive the compiled keep binary: keep spec emits the same OpenAPI inventory as http::routes::openapi (the /openapi.json source), keep spec gen --lang ts writes client files to a temp dir, and keep dockerfile render --variant source|release reproduces the committed Dockerfiles (with keep@version substitution)."
//! End-to-end deploy-CLI surface driven against the COMPILED `keep` binary:
//! `keep spec` emits the same OpenAPI document the server mounts at
//! `/openapi.json`, `keep spec gen` writes a typed client, and `keep dockerfile
//! render` reproduces the committed Dockerfiles. Offline: no server, no network.
//!
//! @spec projects/keep/tech-design/interfaces/cli/deploy-cli-keep-spec-spec-gen-dockerfile-render.md

use std::process::Command;

use serde_json::Value;

fn stdout(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keep"))
        .args(args)
        .output()
        .expect("run keep binary");
    assert!(
        out.status.success(),
        "`keep {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// R1 / AC1: `keep spec` is the exact document the server serves at
/// `/openapi.json` (both come from `http::routes::openapi`) — same inventory,
/// never drifting.
#[test]
fn spec_matches_the_served_openapi_document() {
    let cli: Value = serde_json::from_str(&stdout(&["spec"])).expect("keep spec is JSON");
    let served: Value =
        serde_json::from_str(&keep::spec::openapi_json()).expect("accessor doc is JSON");
    assert_eq!(
        cli, served,
        "keep spec output == the /openapi.json accessor doc"
    );

    let paths = cli["paths"].as_object().expect("paths");
    assert!(
        paths.keys().any(|p| p.contains("/v1/kv")),
        "KV data plane present"
    );
    assert!(
        paths.contains_key("/openapi.json"),
        "the OpenAPI probe path is in the inventory"
    );
}

#[test]
fn spec_format_and_view_variants_render() {
    assert!(stdout(&["spec", "--format", "openapi-yaml"]).contains("openapi:"));
    let js: Value = serde_json::from_str(&stdout(&["spec", "--format", "json-schema"]))
        .expect("json-schema is JSON");
    assert!(js["components"]["schemas"].is_object());
    let shapes: Value =
        serde_json::from_str(&stdout(&["spec", "--shapes"])).expect("shapes is JSON");
    assert!(shapes["shapes"].is_array());
    let fields: Value =
        serde_json::from_str(&stdout(&["spec", "--fields"])).expect("fields is JSON");
    assert!(fields["value_types"].is_array());
}

/// R2 / AC2: `keep spec gen --lang <l> --out <dir>` writes a client for every
/// language via the shared codegen.
#[test]
fn spec_gen_writes_a_client_for_every_language() {
    for lang in ["ts", "py", "rust"] {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join(lang);
        let _ = stdout(&[
            "spec",
            "gen",
            "--lang",
            lang,
            "--out",
            out.to_str().unwrap(),
        ]);
        let files: Vec<_> = std::fs::read_dir(&out)
            .expect("client output dir")
            .filter_map(|e| e.ok())
            .collect();
        assert!(!files.is_empty(), "{lang} client emitted files");
    }
    // The TypeScript client carries the well-known entry files.
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("ts");
    let _ = stdout(&[
        "spec",
        "gen",
        "--lang",
        "ts",
        "--out",
        out.to_str().unwrap(),
    ]);
    for f in ["types.ts", "client.ts", "index.ts"] {
        assert!(out.join(f).is_file(), "generated {f}");
    }
}

/// R3 / AC3 (source): `keep dockerfile render --variant source` reproduces the
/// committed `Dockerfile` byte-for-byte.
#[test]
fn dockerfile_render_source_reproduces_committed_dockerfile() {
    let rendered = stdout(&["dockerfile", "render", "--variant", "source"]);
    assert_eq!(
        rendered,
        include_str!("../Dockerfile"),
        "keep dockerfile render --variant source == committed Dockerfile"
    );
}

/// R3 / AC3 (release): the default render reproduces the committed
/// `Dockerfile.release`, and an explicit `--version` flows into the ARG + tag.
#[test]
fn dockerfile_render_release_reproduces_committed_and_substitutes_version() {
    let rendered = stdout(&["dockerfile", "render", "--variant", "release"]);
    assert_eq!(
        rendered,
        include_str!("../Dockerfile.release"),
        "keep dockerfile render --variant release == committed Dockerfile.release"
    );

    let pinned = stdout(&[
        "dockerfile",
        "render",
        "--variant",
        "release",
        "--version",
        "9.9.9",
    ]);
    assert!(
        pinned.contains("ARG KEEP_VERSION=keep@9.9.9"),
        "pinned ARG: {pinned}"
    );
    assert!(
        pinned.contains("-t keep:9.9.9"),
        "pinned image tag: {pinned}"
    );
}
// HANDWRITE-END
