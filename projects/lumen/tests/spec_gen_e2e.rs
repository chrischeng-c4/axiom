//! `lumen spec gen` — generate a typed client (ts/py/rust) from lumen's own
//! OpenAPI document, offline.
//!
//! @spec projects/lumen/tech-design/interfaces/cli/lumen-spec-gen-generate-a-typed-client-ts-py-rust-from-lumen-s-o.md

use std::process::Command;

fn lumen() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lumen"))
}

/// R1: `spec gen --lang py` writes pydantic + generated sync/async HTTP/2 runtime.
#[test]
fn gen_py_writes_pydantic_h2c_client() {
    let dir = tempfile::tempdir().unwrap();
    let status = lumen()
        .args(["spec", "gen", "--lang", "py", "--out"])
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "spec gen --lang py failed");

    for f in ["models.py", "h2c_runtime.py", "client.py", "__init__.py"] {
        assert!(dir.path().join(f).exists(), "missing {f}");
    }
    let models = std::fs::read_to_string(dir.path().join("models.py")).unwrap();
    assert!(models.contains("BaseModel"), "models.py not pydantic");
    assert!(models.contains("class "), "models.py has no model class");
    let runtime = std::fs::read_to_string(dir.path().join("h2c_runtime.py")).unwrap();
    assert!(
        runtime.contains("class H2CClient"),
        "runtime missing H2CClient"
    );
    assert!(
        runtime.contains("class H2CConnection"),
        "runtime missing connection/session layer"
    );
    assert!(
        runtime.contains("class AsyncH2CClient"),
        "runtime missing async client"
    );
    assert!(runtime.contains("TLS ALPN h2"), "runtime missing ALPN path");
    assert!(
        runtime.contains("def stream("),
        "runtime missing bidi stream surface"
    );
    let client = std::fs::read_to_string(dir.path().join("client.py")).unwrap();
    assert!(
        client.contains("H2CClient"),
        "client.py not wired to generated HTTP/2 runtime"
    );
    assert!(
        client.contains("class AsyncClient"),
        "client.py missing async typed client"
    );
    assert!(
        client.contains("class SupportsRequest"),
        "client.py missing httpx-compatible injection protocol"
    );
}

/// R2: `--lang` selects the emitter (ts → .ts set, rust → .rs set).
#[test]
fn gen_lang_selects_emitter() {
    for (lang, marker) in [("ts", "client.ts"), ("rust", "client.rs")] {
        let dir = tempfile::tempdir().unwrap();
        let status = lumen()
            .args(["spec", "gen", "--lang", lang, "--out"])
            .arg(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "spec gen --lang {lang} failed");
        assert!(dir.path().join(marker).exists(), "{lang}: missing {marker}");
    }
}

/// R3: `lumen spec` (no subcommand) still prints the OpenAPI document unchanged.
#[test]
fn plain_spec_still_prints_openapi() {
    let out = lumen().arg("spec").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.trim_start().starts_with('{'));
    assert!(stdout.contains("\"openapi\""));
}
