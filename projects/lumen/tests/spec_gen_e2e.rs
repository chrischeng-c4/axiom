// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:dc8df8ae" tracker="standardize-gap-projects-lumen-tests-spec-gen-e2e-rs" reason="lumen spec gen e2e: drives the CLI to emit typed clients from lumen's own OpenAPI offline (py pydantic + sync/async h2c runtime, --lang emitter selection) plus plain `spec` OpenAPI passthrough. Not yet captured as unit-test units in lumen-tests.md; aw claim_code/fillback adoption hangs."
//! `lumen spec gen` — generate a typed client (ts/py/rust) from lumen's own
//! OpenAPI document, offline.
//!
//! @spec projects/lumen/tech-design/interfaces/cli/lumen-spec-gen-generate-a-typed-client-ts-py-rust-from-lumen-s-o.md

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

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
    assert!(
        models.contains("RootModel"),
        "models.py missing pydantic RootModel for oneOf component unions"
    );
    assert!(models.contains("class "), "models.py has no model class");
    assert!(
        models.contains("class QueryNodeAnd(BaseModel):"),
        "QueryNode oneOf variants are not pydantic models"
    );
    assert!(
        models.contains("and_: list[QueryNode] = Field(alias=\"and\")"),
        "QueryNode keyword variant alias was not preserved"
    );
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

/// R4: generated Python h2c client drives a live Lumen public API journey.
#[test]
#[ignore = "AW EC gate: opens a local h2c listener; run explicitly with --ignored"]
fn generated_client_live_h2c_public_api_journey() {
    let dir = tempfile::tempdir().unwrap();
    let package_dir = dir.path().join("lumen_client");
    std::fs::create_dir(&package_dir).unwrap();

    let status = lumen()
        .args(["spec", "gen", "--lang", "py", "--out"])
        .arg(&package_dir)
        .status()
        .unwrap();
    assert!(status.success(), "spec gen --lang py failed");

    let port = free_local_port();
    let mut child = ChildGuard::spawn(port);

    let script = dir.path().join("generated_client_live_smoke.py");
    std::fs::write(&script, GENERATED_CLIENT_LIVE_SMOKE).unwrap();
    let output = Command::new("python3")
        .arg(&script)
        .arg(format!("http://127.0.0.1:{port}"))
        .env("PYTHONPATH", dir.path())
        .output()
        .unwrap();

    child.stop();
    assert!(
        output.status.success(),
        "generated Python client live smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn free_local_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn spawn(port: u16) -> Self {
        let child = lumen()
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--wal",
                "embedded",
                "--log-level",
                "warn",
            ])
            .env("LUMEN_AUTH", "off")
            .env("LUMEN_LOG_FORMAT", "json")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        Self { child }
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

const GENERATED_CLIENT_LIVE_SMOKE: &str = r#"
import sys
import time

from lumen_client import (
    Client,
    CreateCollectionRequest,
    DuplicatesRequest,
    FieldSpec,
    IndexItem,
    IndexRequest,
    QueryNode,
    SearchRequest,
)


def wait_ready(client: Client) -> None:
    last = None
    for _ in range(80):
        try:
            client.healthz()
            client.readyz()
            return
        except Exception as exc:
            last = exc
            time.sleep(0.1)
    raise RuntimeError(f"lumen did not become ready: {last!r}")


base_url = sys.argv[1]
collection_id = "generated_client_smoke"

with Client(base_url) as client:
    wait_ready(client)
    assert client.version() is not None
    client.metrics()

    try:
        client.drop_collection(collection_id=collection_id, force=True)
    except Exception:
        pass

    created = client.create_collection(
        collection_id=collection_id,
        body=CreateCollectionRequest(
            fields={
                "body": FieldSpec(type="text"),
                "email": FieldSpec(type="keyword"),
            }
        ),
    )
    assert created.collection_id == collection_id

    indexed = client.index(
        collection_id=collection_id,
        body=IndexRequest(
            items=[
                IndexItem(external_id="u1", field="body", value="blue search one"),
                IndexItem(external_id="u1", field="email", value="dup@example.test"),
                IndexItem(external_id="u2", field="body", value="green search two"),
                IndexItem(external_id="u2", field="email", value="dup@example.test"),
            ]
        ),
    )
    assert indexed.indexed == 4

    query = QueryNode.model_validate({"match": {"field": "body", "text": "blue", "op": "or"}})
    search = client.search(
        collection_id=collection_id,
        body=SearchRequest(query=query, limit=10, track_total=True),
    )
    assert [hit.external_id for hit in search.hits] == ["u1"], search

    dupes = client.duplicates(
        collection_id=collection_id,
        body=DuplicatesRequest(field="email", min_group_size=2),
    )
    assert dupes.groups and dupes.groups[0].external_ids == ["u1", "u2"], dupes

    stats = client.stats(collection_id=collection_id)
    assert stats.fields["body"].type == "text"

    client.drop_collection(collection_id=collection_id, force=True)
"#;
// HANDWRITE-END
