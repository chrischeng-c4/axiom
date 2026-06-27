// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-generated-clients
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:generated-client-journey" tracker="pending-tracker" reason="EC gate executes generated Python/TypeScript/Rust clients against a live h2c Lumen service."
// AW-EC-BEGIN
// @ec lumen-cli-interface-generated-clients
// @capability cli-interface
// @claim lumen-spec-schema-openapi-json-yaml-json-schema-offline
// @contract spec-gen-generated-clients-public-api-journey
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test behavior_lumen_cli_interface_generated_clients -- --ignored --nocapture
// AW-EC-END

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const PY_JOURNEY: &str = r#"
import importlib.util
import pathlib
import sys

pkg = pathlib.Path(sys.argv[1])
base_url = sys.argv[2].rstrip("/")
collection = sys.argv[3]

spec = importlib.util.spec_from_file_location(
    "generated_lumen",
    pkg / "__init__.py",
    submodule_search_locations=[str(pkg)],
)
if spec is None or spec.loader is None:
    raise RuntimeError("could not load generated Python package")
mod = importlib.util.module_from_spec(spec)
sys.modules["generated_lumen"] = mod
spec.loader.exec_module(mod)

with mod.Client(base_url) as client:
    client.healthz()
    client.readyz()
    version = client.version()
    assert isinstance(version, dict) and "version" in version, version

    client.create_collection(
        collection_id=collection,
        body=mod.CreateCollectionRequest(
            fields={
                "email": mod.FieldSpec(type="keyword"),
                "age": mod.FieldSpec(type="number"),
                "bio": mod.FieldSpec(type="text"),
            }
        ),
    )

    indexed = client.index(
        collection_id=collection,
        body=mod.IndexRequest(
            items=[
                mod.IndexItem(external_id="u1", field="email", value="a@x.com"),
                mod.IndexItem(external_id="u1", field="age", value=30),
                mod.IndexItem(external_id="u1", field="bio", value="senior rust search engineer"),
                mod.IndexItem(external_id="u2", field="email", value="b@y.com"),
                mod.IndexItem(external_id="u2", field="age", value=22),
                mod.IndexItem(external_id="u2", field="bio", value="junior frontend engineer"),
                mod.IndexItem(external_id="u3", field="email", value="a@x.com"),
                mod.IndexItem(external_id="u3", field="age", value=28),
                mod.IndexItem(external_id="u3", field="bio", value="product designer"),
            ]
        ),
    )
    assert indexed.indexed == 9, indexed

    query = mod.QueryNode.model_validate(
        {
            "and": [
                {"term": {"field": "email", "value": "a@x.com"}},
                {"not": {"term": {"field": "email", "value": "b@y.com"}}},
            ]
        }
    )
    assert isinstance(query.root, mod.QueryNodeAnd), type(query.root)
    search = client.search(
        collection_id=collection,
        body=mod.SearchRequest(query=query, limit=10, track_total=True),
    )
    ids = sorted(hit.external_id for hit in search.hits)
    assert search.total == 2, search
    assert ids == ["u1", "u3"], ids

    duplicates = client.duplicates(
        collection_id=collection,
        body=mod.DuplicatesRequest(field="email", min_group_size=2, limit=10),
    )
    group = next((g for g in duplicates.groups if g.value == "a@x.com"), None)
    assert group is not None, duplicates
    assert sorted(group.external_ids) == ["u1", "u3"], group

    stats = client.stats(collection_id=collection)
    assert stats.documents_indexed == 3, stats
    assert stats.fields["email"].type == "keyword", stats

    client.drop_collection(collection_id=collection, force=True)
"#;

const TS_JOURNEY: &str = r#"
import { createClient } from "./client";
import type { QueryNode } from "./types";

declare const process: { argv: string[]; exitCode?: number };

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

async function main(): Promise<void> {
  const baseUrl = process.argv[2];
  const collection_id = process.argv[3];
  const client = createClient({ baseUrl });

  await client.healthz();
  await client.readyz();
  const version = await client.version();
  assert(typeof version === "object" && version !== null, `bad version: ${String(version)}`);

  await client.createCollection({
    path: { collection_id },
    body: {
      fields: {
        email: { type: "keyword" },
        age: { type: "number" },
        bio: { type: "text" },
      },
    },
  });

  const indexed = await client.index({
    path: { collection_id },
    body: {
      items: [
        { external_id: "u1", field: "email", value: "a@x.com" },
        { external_id: "u1", field: "age", value: 30 },
        { external_id: "u1", field: "bio", value: "senior rust search engineer" },
        { external_id: "u2", field: "email", value: "b@y.com" },
        { external_id: "u2", field: "age", value: 22 },
        { external_id: "u2", field: "bio", value: "junior frontend engineer" },
        { external_id: "u3", field: "email", value: "a@x.com" },
        { external_id: "u3", field: "age", value: 28 },
        { external_id: "u3", field: "bio", value: "product designer" },
      ],
    },
  });
  assert(indexed.indexed === 9, `bad indexed count: ${JSON.stringify(indexed)}`);

  const query: QueryNode = {
    and: [
      { term: { field: "email", value: "a@x.com" } },
      { not: { term: { field: "email", value: "b@y.com" } } },
    ],
  };
  const search = await client.search({
    path: { collection_id },
    body: { query, limit: 10, track_total: true },
  });
  const ids = search.hits.map((hit) => hit.external_id).sort();
  assert(search.total === 2, `bad search total: ${JSON.stringify(search)}`);
  assert(JSON.stringify(ids) === JSON.stringify(["u1", "u3"]), `bad search ids: ${JSON.stringify(ids)}`);

  const duplicates = await client.duplicates({
    path: { collection_id },
    body: { field: "email", min_group_size: 2, limit: 10 },
  });
  const group = duplicates.groups.find((item) => item.value === "a@x.com");
  assert(group !== undefined, `missing duplicate group: ${JSON.stringify(duplicates)}`);
  assert(
    JSON.stringify([...group.external_ids].sort()) === JSON.stringify(["u1", "u3"]),
    `bad duplicate ids: ${JSON.stringify(group)}`,
  );

  const stats = await client.stats({ path: { collection_id } });
  assert(stats.documents_indexed === 3, `bad stats: ${JSON.stringify(stats)}`);
  assert(stats.fields.email.type === "keyword", `bad email stats: ${JSON.stringify(stats)}`);

  await client.dropCollection({ path: { collection_id }, query: { force: true } });
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
"#;

const RS_CARGO_TOML: &str = r#"
[package]
name = "lumen-generated-client-journey"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
name = "generated_lumen"
path = "mod.rs"

[[bin]]
name = "journey"
path = "journey.rs"

[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#;

const RS_JOURNEY: &str = r#"
use generated_lumen::client::Client;
use generated_lumen::models::*;
use serde_json::json;
use std::collections::HashMap;

fn field(type_: &str) -> FieldSpec {
    FieldSpec {
        analyzer: None,
        backend: None,
        dim: None,
        metric: None,
        multi: None,
        quantize: None,
        type_: type_.to_string(),
    }
}

fn item(external_id: &str, field: &str, value: serde_json::Value) -> IndexItem {
    IndexItem {
        external_id: external_id.to_string(),
        field: field.to_string(),
        value,
        version: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::args().nth(1).expect("base url");
    let collection = std::env::args().nth(2).expect("collection id");
    let client = Client::new(base_url);

    client.healthz()?;
    client.readyz()?;
    let version = client.version()?;
    assert!(version.get("version").is_some(), "bad version: {version}");

    client.create_collection(
        collection.clone(),
        CreateCollectionRequest {
            fields: HashMap::from([
                ("email".to_string(), field("keyword")),
                ("age".to_string(), field("number")),
                ("bio".to_string(), field("text")),
            ]),
        },
    )?;

    let indexed = client.index(
        collection.clone(),
        IndexRequest {
            items: vec![
                item("u1", "email", json!("a@x.com")),
                item("u1", "age", json!(30.0)),
                item("u1", "bio", json!("senior rust search engineer")),
                item("u2", "email", json!("b@y.com")),
                item("u2", "age", json!(22.0)),
                item("u2", "bio", json!("junior frontend engineer")),
                item("u3", "email", json!("a@x.com")),
                item("u3", "age", json!(28.0)),
                item("u3", "bio", json!("product designer")),
            ],
            request_id: None,
        },
    )?;
    assert_eq!(indexed.indexed, 9, "bad indexed count: {indexed:?}");

    let search = client.search(
        collection.clone(),
        SearchRequest {
            collapse: None,
            cursor: None,
            limit: Some(10),
            query: json!({
                "and": [
                    { "term": { "field": "email", "value": "a@x.com" } },
                    { "not": { "term": { "field": "email", "value": "b@y.com" } } }
                ]
            }),
            sort: None,
            track_total: Some(true),
        },
    )?;
    let mut ids = search
        .hits
        .iter()
        .map(|hit| hit.external_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    assert_eq!(search.total, 2, "bad search total: {search:?}");
    assert_eq!(ids, ["u1".to_string(), "u3".to_string()], "bad search ids");

    let duplicates = client.duplicates(
        collection.clone(),
        DuplicatesRequest {
            field: "email".to_string(),
            limit: Some(10),
            min_group_size: Some(2),
            offset: None,
        },
    )?;
    let group = duplicates
        .groups
        .iter()
        .find(|group| group.value == json!("a@x.com"))
        .expect("missing duplicate group");
    let mut duplicate_ids = group.external_ids.clone();
    duplicate_ids.sort();
    assert_eq!(duplicate_ids, ["u1".to_string(), "u3".to_string()]);

    let stats = client.stats(collection.clone())?;
    assert_eq!(stats.documents_indexed, 3, "bad stats: {stats:?}");
    assert_eq!(
        stats.fields.get("email").map(|field| field.type_.as_str()),
        Some("keyword"),
        "bad email stats: {stats:?}",
    );

    client.drop_collection(collection, Some(true))?;
    Ok(())
}
"#;

struct LiveLumen {
    base_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

impl Drop for LiveLumen {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.task.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "AW EC gate: requires python3+pydantic, node+tsc, and nested cargo"]
async fn lumen_cli_interface_generated_clients() {
    let server = spawn_lumen().await;
    let work = tempfile::tempdir().expect("tempdir");

    let py_dir = work.path().join("py");
    gen_client("py", &py_dir, &[]);
    run_python(&py_dir, &server.base_url, "generated_py");

    let ts_dir = work.path().join("ts");
    gen_client("ts", &ts_dir, &["--http", "fetch"]);
    run_typescript(&ts_dir, &server.base_url, "generated_ts");

    let rs_dir = work.path().join("rust");
    gen_client("rust", &rs_dir, &[]);
    run_rust(&rs_dir, &server.base_url, "generated_rs");
}

async fn spawn_lumen() -> LiveLumen {
    let engine = Arc::new(lumen::storage::Engine::new());
    let app = lumen::api::router(lumen::api::AppState::open(engine));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind lumen test server");
    let addr = listener.local_addr().expect("local addr");
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        h2c::serve(listener, app, async move {
            let _ = shutdown_rx.await;
        })
        .await;
    });
    let base_url = format!("http://{addr}");
    wait_ready(&base_url).await;
    LiveLumen {
        base_url,
        shutdown: Some(shutdown_tx),
        task,
    }
}

async fn wait_ready(base_url: &str) {
    let client = reqwest::Client::new();
    let mut last = String::new();
    for _ in 0..50 {
        match client.get(format!("{base_url}/readyz")).send().await {
            Ok(resp) if resp.status().is_success() => return,
            Ok(resp) => last = format!("HTTP {}", resp.status()),
            Err(err) => last = err.to_string(),
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("lumen test server did not become ready: {last}");
}

fn gen_client(lang: &str, out: &Path, extra: &[&str]) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lumen"));
    cmd.args(["spec", "gen", "--lang", lang]);
    cmd.args(extra);
    cmd.arg("--out").arg(out);
    run(&mut cmd, &format!("lumen spec gen --lang {lang}"));
}

fn run_python(dir: &Path, base_url: &str, collection: &str) {
    let script = dir.join("journey.py");
    fs::write(&script, PY_JOURNEY).expect("write Python journey");
    let mut cmd = Command::new("python3");
    cmd.arg(&script).arg(dir).arg(base_url).arg(collection);
    run(&mut cmd, "generated Python client journey");
}

fn run_typescript(dir: &Path, base_url: &str, collection: &str) {
    fs::write(dir.join("journey.ts"), TS_JOURNEY).expect("write TypeScript journey");
    let mut tsc = Command::new("tsc");
    tsc.current_dir(dir).args([
        "--strict",
        "--target",
        "ES2020",
        "--module",
        "CommonJS",
        "--moduleResolution",
        "node",
        "--lib",
        "ES2020,DOM",
        "--skipLibCheck",
        "--outDir",
        "dist",
        "types.ts",
        "runtime.ts",
        "client.ts",
        "journey.ts",
    ]);
    run(&mut tsc, "generated TypeScript client compile");

    let mut node = Command::new("node");
    node.current_dir(dir)
        .arg("dist/journey.js")
        .arg(base_url)
        .arg(collection);
    run(&mut node, "generated TypeScript client journey");
}

fn run_rust(dir: &Path, base_url: &str, collection: &str) {
    fs::write(dir.join("Cargo.toml"), RS_CARGO_TOML).expect("write Rust Cargo.toml");
    fs::write(dir.join("journey.rs"), RS_JOURNEY).expect("write Rust journey");

    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(dir)
        .args(["run", "--offline", "--quiet", "--bin", "journey", "--"])
        .arg(base_url)
        .arg(collection);
    run(&mut cargo, "generated Rust client journey");
}

fn run(cmd: &mut Command, label: &str) {
    let debug = format!("{cmd:?}");
    let output = cmd
        .output()
        .unwrap_or_else(|err| panic!("{label}: failed to spawn {debug}: {err}"));
    assert!(
        output.status.success(),
        "{label} failed: {debug}\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
// HANDWRITE-END
