---
id: projects-lumen-tests-generated-clients-crud-e2e-rs
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "lumen-spec-schema-openapi-json-yaml-json-schema-offline"
    coverage: full
    rationale: "This source unit backs EC contract `spec-gen-generated-clients-public-api-journey` for the Lumen CLI/API interface."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/tests/generated_clients_crud_e2e.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/tests/generated_clients_crud_e2e.rs` generated from AST during Lumen AW health remediation.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-tests-generated_clients_crud_e2e-rs.md#rust-source-unit
// CODEGEN-BEGIN
// @contract spec-gen-generated-clients-public-api-journey
//! Generated-client delivery gate for Lumen itself.
//!
//! This is intentionally under `projects/lumen/tests`, not only `examples/`:
//! Lumen's own test surface is the release guarantee that generated Python,
//! TypeScript, and Rust clients can drive the public API happy path against a
//! real Lumen server.

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cclab_openapi_codegen::{generate, GenOptions, HttpClient, Lang};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct LumenServer {
    base_url: String,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-tests-generated_clients_crud_e2e-rs.md#source
impl LumenServer {
    async fn start() -> Self {
        let engine = Arc::new(lumen::storage::Engine::new());
        let app = lumen::api::router(lumen::api::AppState::open(engine));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind generated-client e2e lumen server");
        let addr: SocketAddr = listener.local_addr().expect("local addr");
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn(async move {
            service_http::serve(listener, app, async {
                let _ = rx.await;
            })
            .await;
        });
        let server = Self {
            base_url: format!("http://{addr}"),
            shutdown: Some(tx),
            handle,
        };
        server.wait_ready().await;
        server
    }

    async fn wait_ready(&self) {
        let client = reqwest::Client::new();
        let url = format!("{}/collections", self.base_url);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    return;
                }
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "generated-client e2e lumen server did not become ready at {url}"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        let _ = tokio::time::timeout(Duration::from_secs(6), self.handle).await;
    }
}

fn opts(lang: Lang) -> GenOptions {
    GenOptions {
        lang,
        spec_path: PathBuf::new(),
        out_dir: PathBuf::new(),
        client_name: "createLumenClient".to_string(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: lang == Lang::Ts,
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let serial = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lumen-generated-client-crud-{label}-{}-{nanos}-{serial}",
        std::process::id()
    ))
}

fn write_output(dir: &Path, output: cclab_openapi_codegen::GeneratedOutput) {
    fs::create_dir_all(dir).expect("create generated output dir");
    for file in output.files {
        let path = dir.join(file.rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create generated parent dir");
        }
        fs::write(path, file.contents).expect("write generated file");
    }
}

fn command_exists(binary: &str) -> bool {
    Command::new(binary)
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn generated_python_typescript_and_rust_clients_run_crud_happypath() {
    let server = LumenServer::start().await;
    generated_python_client_runs_crud_happypath(&server.base_url);
    generated_typescript_client_runs_crud_happypath(&server.base_url);
    generated_rust_client_runs_crud_happypath(&server.base_url);
    server.shutdown().await;
}

fn generated_python_client_runs_crud_happypath(base_url: &str) {
    if !command_exists("python3") {
        eprintln!("skip python generated-client CRUD e2e: python3 not found");
        return;
    }
    let pydantic = Command::new("python3")
        .arg("-c")
        .arg("import pydantic")
        .status()
        .expect("run python3");
    if !pydantic.success() {
        eprintln!("skip python generated-client CRUD e2e: pydantic not installed");
        return;
    }

    let dir = unique_temp_dir("py");
    let pkg = dir.join("generated_api");
    write_output(
        &pkg,
        generate(&lumen::spec::openapi_json(), &opts(Lang::Py)).unwrap(),
    );
    let script = format!(
        r#"
import sys
sys.path.insert(0, {dir:?})
from generated_api import (
    Client,
    CreateCollectionRequest,
    FieldSpec,
    IndexItem,
    IndexRequest,
    QueryNode,
    QueryNodeTerm,
    SearchRequest,
    TermQuery,
)
with Client({base_url:?}) as client:
    assert client.list_collections() == []
    created = client.create_collection(
        collection_id="users",
        body=CreateCollectionRequest(fields={{"email": FieldSpec(type="keyword")}}),
    )
    assert created.collection_id == "users", created
    indexed = client.index(
        collection_id="users",
        body=IndexRequest(items=[
            IndexItem(external_id="u1", field="email", value="a@x.com"),
            IndexItem(external_id="u2", field="email", value="a@x.com"),
            IndexItem(external_id="u3", field="email", value="b@y.com"),
        ]),
    )
    assert indexed.indexed == 3, indexed
    query = QueryNode(root=QueryNodeTerm(term=TermQuery(field="email", value="a@x.com")))
    found = client.search(collection_id="users", body=SearchRequest(query=query, limit=10))
    assert found.total == 2, found
    assert [hit.external_id for hit in found.hits] == ["u1", "u2"], found.hits
    stats = client.stats(collection_id="users")
    assert stats.documents_indexed == 3, stats
    client.delete_external_id(collection_id="users", external_id="u1", field="email")
    after_delete = client.search(collection_id="users", body=SearchRequest(query=query, limit=10))
    assert after_delete.total == 1, after_delete
    assert [hit.external_id for hit in after_delete.hits] == ["u2"], after_delete.hits
    client.drop_collection(collection_id="users", force=True)
    assert client.list_collections() == []
"#,
        dir = dir.display().to_string(),
        base_url = base_url,
    );
    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .output()
        .expect("run generated Python client CRUD e2e");
    assert!(
        output.status.success(),
        "generated Python client CRUD e2e failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}

fn generated_typescript_client_runs_crud_happypath(base_url: &str) {
    if !command_exists("node") || !command_exists("tsc") {
        eprintln!("skip TypeScript generated-client CRUD e2e: node or tsc not found");
        return;
    }

    let dir = unique_temp_dir("ts");
    write_output(
        &dir,
        generate(&lumen::spec::openapi_json(), &opts(Lang::Ts)).unwrap(),
    );
    fs::write(
        dir.join("smoke.ts"),
        format!(
            r#"
declare const process: {{ argv: string[]; exit(code?: number): never }};
import {{ createLumenClient }} from "./client";

async function main() {{
  const client = createLumenClient({{
    baseUrl: {base_url:?},
    transport: {{ targetConcurrency: 128, maxInFlightPerOrigin: 128, poolTimeoutMs: 5000 }},
  }});
  const initial = await client.listCollections();
  if (!Array.isArray(initial) || initial.length !== 0) {{
    throw new Error(`unexpected initial collections payload: ${{JSON.stringify(initial)}}`);
  }}
  const created = await client.createCollection({{
    path: {{ collection_id: "users" }},
    body: {{ fields: {{ email: {{ type: "keyword" }} }} }},
  }});
  if (created.collection_id !== "users") {{
    throw new Error(`unexpected create response: ${{JSON.stringify(created)}}`);
  }}
  const indexed = await client.index({{
    path: {{ collection_id: "users" }},
    body: {{
      items: [
        {{ external_id: "u1", field: "email", value: "a@x.com" }},
        {{ external_id: "u2", field: "email", value: "a@x.com" }},
        {{ external_id: "u3", field: "email", value: "b@y.com" }},
      ],
    }},
  }});
  if (indexed.indexed !== 3) {{
    throw new Error(`unexpected index response: ${{JSON.stringify(indexed)}}`);
  }}
  const query = {{ term: {{ field: "email", value: "a@x.com" }} }};
  const found = await client.search({{
    path: {{ collection_id: "users" }},
    body: {{ query, limit: 10 }},
  }});
  if (found.total !== 2 || found.hits.map((hit) => hit.external_id).join(",") !== "u1,u2") {{
    throw new Error(`unexpected search response: ${{JSON.stringify(found)}}`);
  }}
  const stats = await client.stats({{ path: {{ collection_id: "users" }} }});
  if (stats.documents_indexed !== 3) {{
    throw new Error(`unexpected stats response: ${{JSON.stringify(stats)}}`);
  }}
  await client.deleteExternalId({{
    path: {{ collection_id: "users", external_id: "u1" }},
    query: {{ field: "email" }},
  }});
  const afterDelete = await client.search({{
    path: {{ collection_id: "users" }},
    body: {{ query, limit: 10 }},
  }});
  if (afterDelete.total !== 1 || afterDelete.hits.map((hit) => hit.external_id).join(",") !== "u2") {{
    throw new Error(`unexpected search-after-delete response: ${{JSON.stringify(afterDelete)}}`);
  }}
  await client.dropCollection({{
    path: {{ collection_id: "users" }},
    query: {{ force: true }},
  }});
  const finalCollections = await client.listCollections();
  if (finalCollections.length !== 0) {{
    throw new Error(`unexpected final collections payload: ${{JSON.stringify(finalCollections)}}`);
  }}
}}

main().catch((err) => {{
  console.error(err);
  process.exit(1);
}});
"#,
            base_url = base_url,
        ),
    )
    .expect("write TypeScript CRUD e2e");
    let dist = dir.join("dist");
    let tsc = Command::new("tsc")
        .current_dir(&dir)
        .args([
            "--target",
            "ES2022",
            "--module",
            "commonjs",
            "--moduleResolution",
            "node",
            "--lib",
            "ES2022,DOM",
            "--skipLibCheck",
            "--outDir",
            dist.to_str().unwrap(),
            "runtime.ts",
            "types.ts",
            "client.ts",
            "smoke.ts",
        ])
        .output()
        .expect("run tsc for generated TypeScript client CRUD e2e");
    assert!(
        tsc.status.success(),
        "generated TypeScript compile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&tsc.stdout),
        String::from_utf8_lossy(&tsc.stderr)
    );
    let node = Command::new("node")
        .arg(dist.join("smoke.js"))
        .output()
        .expect("run generated TypeScript client CRUD e2e");
    assert!(
        node.status.success(),
        "generated TypeScript client CRUD e2e failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&node.stdout),
        String::from_utf8_lossy(&node.stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}

fn generated_rust_client_runs_crud_happypath(base_url: &str) {
    if !command_exists("cargo") {
        eprintln!("skip Rust generated-client CRUD e2e: cargo not found");
        return;
    }

    let dir = unique_temp_dir("rs");
    let src = dir.join("src");
    write_output(
        &src,
        generate(&lumen::spec::openapi_json(), &opts(Lang::Rust)).unwrap(),
    );
    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "generated-lumen-rust-client-smoke"
version = "0.0.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls-native-roots"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#,
    )
    .expect("write generated Rust CRUD e2e Cargo.toml");
    fs::write(
        src.join("main.rs"),
        format!(
            r#"
mod client;
mod models;

use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let client = client::Client::new({base_url:?});
    assert!(client.list_collections()?.is_empty());

    let mut fields = HashMap::new();
    fields.insert(
        "email".to_string(),
        models::FieldSpec {{
            analyzer: None,
            backend: None,
            dim: None,
            metric: None,
            multi: None,
            quantize: None,
            type_: "keyword".to_string(),
        }},
    );
    let created = client.create_collection(
        "users".to_string(),
        models::CreateCollectionRequest {{ fields }},
    )?;
    assert_eq!(created.collection_id, "users");

    let indexed = client.index(
        "users".to_string(),
        models::IndexRequest {{
            items: vec![
                models::IndexItem {{ external_id: "u1".to_string(), field: "email".to_string(), value: serde_json::json!("a@x.com"), version: None }},
                models::IndexItem {{ external_id: "u2".to_string(), field: "email".to_string(), value: serde_json::json!("a@x.com"), version: None }},
                models::IndexItem {{ external_id: "u3".to_string(), field: "email".to_string(), value: serde_json::json!("b@y.com"), version: None }},
            ],
            request_id: None,
        }},
    )?;
    assert_eq!(indexed.indexed, 3);

    let query = serde_json::json!({{"term": {{"field": "email", "value": "a@x.com"}}}});
    let found = client.search(
        "users".to_string(),
        models::SearchRequest {{
            collapse: None,
            cursor: None,
            limit: Some(10),
            query: query.clone(),
            routing_key: None,
            sort: None,
            track_total: None,
        }},
    )?;
    assert_eq!(found.total, 2);
    assert_eq!(
        found.hits.iter().map(|hit| hit.external_id.as_str()).collect::<Vec<_>>(),
        vec!["u1", "u2"]
    );

    let stats = client.stats("users".to_string())?;
    assert_eq!(stats.documents_indexed, 3);

    client.delete_external_id("users".to_string(), "u1".to_string(), Some("email".to_string()))?;
    let after_delete = client.search(
        "users".to_string(),
        models::SearchRequest {{
            collapse: None,
            cursor: None,
            limit: Some(10),
            query,
            routing_key: None,
            sort: None,
            track_total: None,
        }},
    )?;
    assert_eq!(after_delete.total, 1);
    assert_eq!(
        after_delete.hits.iter().map(|hit| hit.external_id.as_str()).collect::<Vec<_>>(),
        vec!["u2"]
    );

    client.drop_collection("users".to_string(), Some(true))?;
    assert!(client.list_collections()?.is_empty());
    Ok(())
}}
"#,
            base_url = base_url,
        ),
    )
    .expect("write generated Rust CRUD e2e main");
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(&dir)
        .output()
        .expect("run generated Rust client CRUD e2e");
    assert!(
        output.status.success(),
        "generated Rust client CRUD e2e failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "projects/lumen/tests/generated_clients_crud_e2e.rs"
    action: modify
    section: rust-source-unit
    description: |
      Generated-client CRUD e2e source is captured as a codegen replay unit while retaining the EC contract marker.
    impl_mode: codegen
```
