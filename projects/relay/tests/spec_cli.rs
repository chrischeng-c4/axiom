// HANDWRITE-BEGIN gap="missing-generator:unit-test:be8fe7b3" tracker="pending-tracker" reason="Drives the COMPILED relay binary offline (deploy_cli.rs harness pattern): spec parses as OpenAPI JSON listing the /v1 paths + /admin/backup; --format openapi-yaml parses via serde_yaml; --format json-schema parses as JSON with a components key; spec gen writes a non-empty client per lang (ts asserts types.ts/client.ts/index.ts); llm operations names the new surfaces."
//! Offline `relay spec` surface driven against the COMPILED `relay` binary
//! (WI #1209, keep #777 pattern): every `--format` emits a parseable document
//! that matches the served `/openapi.json` inventory (including the new
//! `/admin/backup`), `spec gen` writes a non-empty typed client per language
//! through the shared `cclab-openapi-codegen`, and the `llm` operations topic
//! names the backup / peer-TLS / spec surfaces. No server, no network.

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_relay"))
        .args(args)
        .output()
        .expect("run relay binary")
}

fn stdout(args: &[&str]) -> String {
    let out = run(args);
    assert!(
        out.status.success(),
        "`relay {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// R1 / AC1: `relay spec` emits parseable OpenAPI in every format — the
/// offline twin of `GET /openapi.json`, listing the /v1 data plane and the
/// new `/admin/backup`. `--format json-schema` carries the `components` key
/// honestly (relay registers no named schemas today, so it may be null —
/// never a faked catalog; keep's `--shapes`/`--fields` are deliberately
/// absent from relay's surface).
#[test]
fn spec_prints_parseable_openapi_in_every_format() {
    // Default: pretty OpenAPI JSON.
    let doc: serde_json::Value =
        serde_json::from_str(&stdout(&["spec"])).expect("`relay spec` emits JSON");
    assert!(doc["openapi"].is_string(), "openapi version field present");
    for path in [
        "/v1/{subject}/publish",
        "/v1/{subject}/consume",
        "/v1/{subject}/lease",
        "/v1/{subject}/len",
        "/admin/backup",
    ] {
        assert!(
            doc["paths"].get(path).is_some(),
            "OpenAPI doc must list {path}"
        );
    }

    // YAML for LLM/agent reading.
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&stdout(&["spec", "--format", "openapi-yaml"]))
            .expect("`relay spec --format openapi-yaml` emits YAML");
    assert!(
        yaml.get("openapi").is_some(),
        "YAML document carries the openapi version field"
    );

    // Component schemas only.
    let schema: serde_json::Value =
        serde_json::from_str(&stdout(&["spec", "--format", "json-schema"]))
            .expect("`relay spec --format json-schema` emits JSON");
    assert!(
        schema.get("components").is_some(),
        "json-schema view carries the components key (null is honest: relay \
         registers no named schemas)"
    );

    // relay has no request-shape/value catalogs: the keep-only flags must NOT
    // parse (omitted, not faked).
    assert!(
        !run(&["spec", "--shapes"]).status.success(),
        "--shapes is keep-only and must not parse on relay"
    );
    assert!(
        !run(&["spec", "--fields"]).status.success(),
        "--fields is keep-only and must not parse on relay"
    );
}

/// R1 / AC1: `relay spec gen --lang <l> --out <dir>` writes a non-empty
/// client for every language via the shared codegen; the TypeScript client
/// carries the well-known entry files.
#[test]
fn spec_gen_writes_a_client_for_every_language() {
    for lang in ["ts", "py", "rust"] {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = dir.path().join(lang);
        let _ = stdout(&["spec", "gen", "--lang", lang, "--out", out.to_str().unwrap()]);
        let files: Vec<_> = std::fs::read_dir(&out)
            .expect("client output dir")
            .filter_map(|e| e.ok())
            .collect();
        assert!(!files.is_empty(), "{lang} client emitted files");
        for f in &files {
            assert!(
                f.metadata().expect("metadata").len() > 0,
                "{lang} client file {:?} is non-empty",
                f.file_name()
            );
        }
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("ts");
    let _ = stdout(&["spec", "gen", "--lang", "ts", "--out", out.to_str().unwrap()]);
    for f in ["types.ts", "client.ts", "index.ts"] {
        assert!(out.join(f).is_file(), "generated {f}");
    }
}

/// R5: the `llm` operations topic documents the spec verbs, the backup verb +
/// endpoint + CronJob token env, and the peer-TLS env contract (with the
/// mTLS-termination seam gap stated, not hidden).
#[test]
fn llm_operations_topic_documents_the_new_surfaces() {
    let ops = stdout(&["llm", "operations"]);
    for needle in [
        "relay spec",
        "spec gen",
        "relay backup",
        "/admin/backup",
        "RELAY_BACKUP_TOKEN",
        "RELAY_PEER_TLS_CERT",
        "RELAY_PEER_MTLS",
    ] {
        assert!(ops.contains(needle), "llm operations must document {needle}");
    }
}
// HANDWRITE-END
