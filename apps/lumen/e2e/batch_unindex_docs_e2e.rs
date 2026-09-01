//! Black-box contract for `POST /collections/{collection_id}/docs:unindex` (#3994).
//!
//! This removes complete indexed rows for a bounded, caller-selected batch of
//! opaque external ids. It is a write-only operation. It accepts no
//! idempotency or request id field because a missing id is already a no-op.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use axum::http::StatusCode;
use axum_test::TestServer;
use openapi_codegen::{generate, GenOptions, HttpClient, Lang};
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState, CheckpointSink};
use lumen::auth::{AuthConfig, LumenVerifier, COLLECTIONS_RESOURCE};
use lumen::coordinator::{SharedAof, WriteCoordinator};
use lumen::segment_rdb::SegmentRdbStore;
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use service_auth::k8s::{
    AccessReviewOutcome, ResourceAttributes, ReviewBackend, ReviewError, ReviewedIdentity,
    TokenReviewOutcome,
};

const COLLECTION: &str = "documents";
const NAMESPACE: &str = "serving";
const ADMIN: &str = "system:serviceaccount:serving:lumen-admin";
const WRITER: &str = "system:serviceaccount:serving:lumen-writer";
const READER: &str = "system:serviceaccount:serving:lumen-reader";
const STRANGER: &str = "system:serviceaccount:serving:lumen-stranger";

fn open_server() -> (TestServer, Arc<Engine>) {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone()))).expect("open server");
    (server, engine)
}

fn all_types_schema() -> Value {
    json!({ "fields": {
        "body": { "type": "text" },
        "state": { "type": "keyword" },
        "rank": { "type": "number" },
        "tags": { "type": "set" },
        "embedding": { "type": "vector", "dim": 2, "metric": "cosine", "backend": "flat-cpu" },
        "fingerprint": { "type": "hash" }
    }})
}

async fn create_all_types_schema(server: &TestServer) {
    server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&all_types_schema())
        .await
        .assert_status_ok();
}

async fn replace_docs(server: &TestServer, docs: Value) {
    server
        .put(&format!("/collections/{COLLECTION}/docs:replace"))
        .json(&json!({ "docs": docs }))
        .await
        .assert_status_ok();
}

async fn documents_indexed(server: &TestServer) -> u64 {
    let response = server
        .get(&format!("/collections/{COLLECTION}/stats"))
        .await;
    response.assert_status_ok();
    response.json::<Value>()["documents_indexed"]
        .as_u64()
        .expect("documents_indexed is numeric")
}

fn snapshot_value(engine: &Engine) -> Value {
    serde_json::to_value(engine.snapshot().expect("snapshot engine")).expect("serialize snapshot")
}

fn assert_snapshot_lacks_external_id(snapshot: &Value, external_id: &str) {
    let encoded = snapshot.to_string();
    assert!(
        !encoded.contains(&format!("\"{external_id}\"")),
        "deleted external id {external_id:?} remained in snapshot: {snapshot}"
    );
}

/// The auth oracle grants exactly the resource tuple required by each role.
struct AuthzOracle {
    tokens: HashMap<&'static str, &'static str>,
    grants: HashSet<(String, String, Option<String>, String)>,
}

impl AuthzOracle {
    fn new() -> Self {
        Self {
            tokens: HashMap::from([
                ("admin", ADMIN),
                ("writer", WRITER),
                ("reader", READER),
                ("stranger", STRANGER),
            ]),
            grants: HashSet::from([
                (
                    ADMIN.to_string(),
                    COLLECTIONS_RESOURCE.to_string(),
                    Some(COLLECTION.to_string()),
                    "delete".to_string(),
                ),
                (
                    WRITER.to_string(),
                    COLLECTIONS_RESOURCE.to_string(),
                    Some(COLLECTION.to_string()),
                    "update".to_string(),
                ),
                (
                    READER.to_string(),
                    COLLECTIONS_RESOURCE.to_string(),
                    Some(COLLECTION.to_string()),
                    "get".to_string(),
                ),
            ]),
        }
    }
}

#[async_trait]
impl ReviewBackend for AuthzOracle {
    async fn review_token(
        &self,
        token: &str,
        audiences: &[String],
    ) -> Result<TokenReviewOutcome, ReviewError> {
        Ok(match self.tokens.get(token) {
            Some(username) => TokenReviewOutcome {
                authenticated: true,
                identity: ReviewedIdentity {
                    username: (*username).to_string(),
                    ..Default::default()
                },
                audiences: audiences.to_vec(),
                error: None,
            },
            None => TokenReviewOutcome {
                authenticated: false,
                identity: ReviewedIdentity::default(),
                audiences: Vec::new(),
                error: Some("unknown token".to_string()),
            },
        })
    }

    async fn review_access(
        &self,
        identity: &ReviewedIdentity,
        attributes: &ResourceAttributes,
    ) -> Result<AccessReviewOutcome, ReviewError> {
        let grant = (
            identity.username.clone(),
            attributes.resource.clone(),
            attributes.name.clone(),
            attributes.verb.clone(),
        );
        Ok(if self.grants.contains(&grant) {
            AccessReviewOutcome::allow()
        } else {
            AccessReviewOutcome::deny("no matching RoleBinding")
        })
    }
}

fn delegated_server() -> TestServer {
    let engine = Arc::new(Engine::new());
    let verifier = Arc::new(
        LumenVerifier::delegated(NAMESPACE, Arc::new(AuthzOracle::new()))
            .expect("delegated verifier"),
    );
    let state =
        AppState::new(engine, Arc::new(AuthConfig::required_in(NAMESPACE))).with_verifier(verifier);
    TestServer::new(router(state)).expect("delegated server")
}

#[tokio::test]
async fn batch_unindex_removes_all_field_types_partial_rows_and_missing_ids_then_allows_rewrite() {
    let (server, engine) = open_server();
    create_all_types_schema(&server).await;
    replace_docs(
        &server,
        json!([
            {
                "external_id": "all-fields",
                "fields": {
                    "body": "remove complete row",
                    "state": "retired",
                    "rank": 7.0,
                    "tags": ["old", "all"],
                    "embedding": [0.0, 1.0],
                    "fingerprint": "0000000000000001"
                }
            },
            {
                "external_id": "survivor",
                "fields": {
                    "body": "keep complete row",
                    "state": "live",
                    "rank": 9.0,
                    "tags": ["keep"],
                    "embedding": [1.0, 0.0],
                    "fingerprint": "0000000000000002"
                }
            }
        ]),
    )
    .await;
    // Partial-index state is deliberately distinct from full docs: the batch
    // operation must remove every field that happens to exist for this id.
    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({ "items": [
            { "external_id": "partial-fields", "field": "body", "value": "remove partial row" },
            { "external_id": "partial-fields", "field": "state", "value": "retired" }
        ]}))
        .await
        .assert_status_ok();
    assert_eq!(documents_indexed(&server).await, 3);

    server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": ["all-fields", "partial-fields", "missing"] }))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    assert_eq!(
        documents_indexed(&server).await,
        1,
        "only the untouched sibling may remain indexed"
    );
    let snapshot = snapshot_value(&engine);
    assert_snapshot_lacks_external_id(&snapshot, "all-fields");
    assert_snapshot_lacks_external_id(&snapshot, "partial-fields");
    assert!(
        snapshot.to_string().contains("\"survivor\""),
        "batch unindex must preserve sibling documents: {snapshot}"
    );

    replace_docs(
        &server,
        json!([
            { "external_id": "all-fields", "fields": { "body": "rewritten", "state": "active" } },
            { "external_id": "partial-fields", "fields": { "body": "rewritten partial", "state": "active" } }
        ]),
    )
    .await;
    assert_eq!(
        documents_indexed(&server).await,
        3,
        "unindexed ids must be usable by a later write"
    );
}

#[tokio::test]
async fn batch_unindex_accepts_an_exactly_1000_id_missing_batch_without_mutation() {
    let (server, _engine) = open_server();
    create_all_types_schema(&server).await;
    replace_docs(
        &server,
        json!([{
            "external_id": "survivor",
            "fields": { "body": "must remain indexed" }
        }]),
    )
    .await;

    let external_ids = (0..1000)
        .map(|index| format!("missing-{index}"))
        .collect::<Vec<_>>();
    server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": external_ids }))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    assert_eq!(
        documents_indexed(&server).await,
        1,
        "an exactly-1000 missing-id batch is a valid no-op"
    );
}

#[tokio::test]
async fn batch_unindex_rejects_invalid_payloads_without_mutation_before_fences() {
    let (server, engine) = open_server();
    create_all_types_schema(&server).await;
    replace_docs(
        &server,
        json!([{ "external_id": "keep", "fields": { "body": "live", "state": "live" } }]),
    )
    .await;
    let before = snapshot_value(&engine);

    for invalid in [
        json!({ "external_ids": [] }),
        json!({ "external_ids": ["duplicate", "duplicate"] }),
        json!({ "external_ids": [42] }),
        json!({ "external_ids": ["keep"], "request_id": "not-supported" }),
        json!({ "external_ids": ["keep"], "unexpected": true }),
        json!(["keep"]),
    ] {
        server
            .post(&format!("/collections/{COLLECTION}/docs:unindex"))
            .json(&invalid)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
        assert_eq!(
            snapshot_value(&engine),
            before,
            "invalid body must not mutate indexed data: {invalid}"
        );
    }

    let over_limit: Vec<Value> = (0..1001)
        .map(|number| json!(format!("id-{number}")))
        .collect();
    server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": over_limit }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(
        snapshot_value(&engine),
        before,
        "over-limit batch must not mutate"
    );

    // The malformed request must fail at JSON validation. An armed write fence
    // must not replace this deterministic 400 with a 503 or route anything.
    server
        .post("/admin/reshard:fence")
        .json(&json!({ "virtual_bucket_count": 4, "buckets": [0], "ttl_secs": 30 }))
        .await
        .assert_status_ok();
    server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": [] }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(
        snapshot_value(&engine),
        before,
        "fenced invalid body must not mutate"
    );
}

#[tokio::test]
async fn batch_unindex_requires_write_role() {
    let server = delegated_server();
    server
        .put(&format!("/collections/{COLLECTION}"))
        .add_header("authorization", "Bearer admin")
        .json(&all_types_schema())
        .await
        .assert_status_ok();
    server
        .put(&format!("/collections/{COLLECTION}/docs:replace"))
        .add_header("authorization", "Bearer writer")
        .json(&json!({ "docs": [{
            "external_id": "delete-me", "fields": { "body": "old", "state": "retired" }
        }]}))
        .await
        .assert_status_ok();

    for token in ["admin", "reader", "stranger"] {
        server
            .post(&format!("/collections/{COLLECTION}/docs:unindex"))
            .add_header("authorization", format!("Bearer {token}"))
            .json(&json!({ "external_ids": ["delete-me"] }))
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }
    server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": ["delete-me"] }))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .add_header("authorization", "Bearer writer")
        .json(&json!({ "external_ids": ["delete-me"] }))
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn batch_unindex_returns_storage_full_without_mutation() {
    let (server, engine) = open_server();
    create_all_types_schema(&server).await;
    replace_docs(
        &server,
        json!([{ "external_id": "keep", "fields": { "body": "live", "state": "live" } }]),
    )
    .await;
    let before = snapshot_value(&engine);
    engine.metrics().mark_storage_degraded();

    let rejected = server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": ["keep"] }))
        .await;
    rejected.assert_status(StatusCode::INSUFFICIENT_STORAGE);
    assert_eq!(rejected.json::<Value>()["error"], "storage_full");
    assert_eq!(
        snapshot_value(&engine),
        before,
        "ENOSPC must not unindex data"
    );
}

struct LocalCheckpointSink {
    engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    writer: Arc<WriteCoordinator>,
    aof: SharedAof,
}

#[async_trait]
impl CheckpointSink for LocalCheckpointSink {
    async fn checkpoint_now(&self) -> Result<bool> {
        let _permit = self.writer.mutation_gate().shared().await?;
        let sequence = self.writer.applied_seq();
        self.store.save(&self.engine, sequence)?;
        self.aof
            .lock()
            .map_err(|_| anyhow::anyhow!("AOF lock poisoned"))?
            .truncate_through(sequence)?;
        Ok(true)
    }
}

struct DurableFixture {
    _temp: tempfile::TempDir,
    server: TestServer,
    engine: Arc<Engine>,
    writer: Arc<WriteCoordinator>,
    aof_path: std::path::PathBuf,
    store: Arc<SegmentRdbStore>,
}

fn durable_fixture() -> DurableFixture {
    let temp = tempfile::tempdir().expect("temporary durable fixture");
    let aof_path = temp.path().join("aof.log");
    let store =
        Arc::new(SegmentRdbStore::new(temp.path().join("segments")).expect("segment store"));
    let aof: SharedAof = Arc::new(Mutex::new(AofWriter::open(&aof_path).expect("AOF")));
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(MemWal::new());
    let writer = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
    let checkpoint = Arc::new(LocalCheckpointSink {
        engine: engine.clone(),
        store: store.clone(),
        writer: writer.clone(),
        aof,
    });
    let state =
        AppState::with_components(engine.clone(), Arc::new(AuthConfig::open()), writer.clone())
            .with_checkpoint(checkpoint);
    DurableFixture {
        _temp: temp,
        server: TestServer::new(router(state)).expect("durable server"),
        engine,
        writer,
        aof_path,
        store,
    }
}

async fn checkpoint(server: &TestServer) {
    let response = server.post("/admin/checkpoint").await;
    response.assert_status_ok();
    assert_eq!(response.json::<Value>()["persisted"], true);
}

#[tokio::test]
async fn batch_unindex_survives_aof_checkpoint_reopen_snapshot_and_reindex() {
    let fixture = durable_fixture();
    fixture
        .server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({ "fields": { "state": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    replace_docs(
        &fixture.server,
        json!([
            { "external_id": "remove", "fields": { "state": "retired" } },
            { "external_id": "keep", "fields": { "state": "live" } }
        ]),
    )
    .await;
    checkpoint(&fixture.server).await;
    let before_unindex_sequence = fixture.writer.applied_seq();

    fixture
        .server
        .post(&format!("/collections/{COLLECTION}/docs:unindex"))
        .json(&json!({ "external_ids": ["remove", "missing"] }))
        .await
        .assert_status(StatusCode::NO_CONTENT);
    assert_eq!(documents_indexed(&fixture.server).await, 1);
    assert!(fixture.writer.applied_seq() > before_unindex_sequence);

    let checkpointed = fixture
        .store
        .load_current_generation()
        .expect("load pre-unindex checkpoint")
        .expect("pre-unindex checkpoint exists");
    let replayed = replay_aof_into(
        &checkpointed.engine,
        &fixture.aof_path,
        checkpointed.sequence,
    )
    .expect("replay unindex AOF tail");
    assert!(
        replayed > checkpointed.sequence,
        "unindex must have an AOF record"
    );
    assert_eq!(
        checkpointed
            .engine
            .stats(COLLECTION)
            .unwrap()
            .documents_indexed,
        1
    );

    checkpoint(&fixture.server).await;
    let sealed = fixture
        .store
        .load_current_generation()
        .expect("load post-unindex checkpoint")
        .expect("post-unindex checkpoint exists");
    assert_eq!(
        sealed.engine.stats(COLLECTION).unwrap().documents_indexed,
        1
    );
    assert_eq!(
        replay_aof_into(&sealed.engine, &fixture.aof_path, sealed.sequence).unwrap(),
        0,
        "post-unindex checkpoint must be self-contained"
    );

    let restored = Engine::new();
    restored
        .restore(fixture.engine.snapshot().expect("live snapshot"))
        .unwrap();
    assert_eq!(restored.stats(COLLECTION).unwrap().documents_indexed, 1);
    restored
        .index(
            COLLECTION,
            lumen::types::IndexRequest {
                items: vec![lumen::types::IndexItem {
                    external_id: "remove".to_string(),
                    field: "state".to_string(),
                    value: lumen::types::FieldValue::String("rewritten".to_string()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .expect("reindex a batch-unindexed id after snapshot restore");
    assert_eq!(restored.stats(COLLECTION).unwrap().documents_indexed, 2);
}

#[test]
fn batch_unindex_openapi_contract_is_exact_and_has_no_request_id() {
    let spec: Value = serde_json::from_str(&lumen::spec::openapi_json()).expect("OpenAPI parses");
    let operation = &spec["paths"]["/collections/{collection_id}/docs:unindex"]["post"];
    assert!(
        operation.is_object(),
        "batch unindex operation missing: {operation}"
    );
    assert_eq!(operation["operationId"], "batch_unindex_docs");
    assert!(
        operation["responses"]["204"].is_object(),
        "batch unindex must return 204"
    );

    let request = &operation["requestBody"]["content"]["application/json"]["schema"];
    assert_eq!(
        request["$ref"],
        "#/components/schemas/BatchUnindexDocsRequest"
    );
    let schema = &spec["components"]["schemas"]["BatchUnindexDocsRequest"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["required"], json!(["external_ids"]));
    assert_eq!(schema["additionalProperties"], false);
    let properties = schema["properties"]
        .as_object()
        .expect("request properties");
    assert_eq!(
        properties.len(),
        1,
        "only external_ids is accepted: {schema}"
    );
    assert!(
        properties.get("request_id").is_none(),
        "request_id is forbidden: {schema}"
    );
    let ids = &properties["external_ids"];
    assert_eq!(ids["type"], "array");
    assert_eq!(ids["minItems"], 1);
    assert_eq!(ids["maxItems"], 1000);
    assert_eq!(ids["uniqueItems"], true);
    assert_eq!(ids["items"]["type"], "string");
    assert!(
        ids["items"].get("format").is_none(),
        "external ids are opaque strings"
    );
}

fn generated_client_options(lang: Lang) -> GenOptions {
    GenOptions {
        lang,
        target: None,
        spec_path: Default::default(),
        out_dir: Default::default(),
        client_name: "createLumenClient".to_string(),
        http_client: HttpClient::Fetch,
        emit_types: true,
        emit_client: true,
        emit_hooks: lang == Lang::Ts,
    }
}

#[test]
fn generated_clients_expose_batch_unindex_docs_and_its_typed_request() {
    for (lang, operation, request_model) in [
        (
            Lang::Ts,
            "batchUnindexDocs(",
            "export interface BatchUnindexDocsRequest",
        ),
        (
            Lang::Py,
            "def batch_unindex_docs(",
            "class BatchUnindexDocsRequest(BaseModel):",
        ),
        (
            Lang::Rust,
            "pub fn batch_unindex_docs(",
            "pub struct BatchUnindexDocsRequest",
        ),
    ] {
        let generated = generate(
            &lumen::spec::openapi_json(),
            &generated_client_options(lang),
        )
        .unwrap_or_else(|error| panic!("generate {lang:?} client: {error}"));
        let output = generated
            .files
            .into_iter()
            .map(|file| file.contents)
            .collect::<Vec<_>>()
            .join("\n");
        for expected in [operation, request_model, "external_ids"] {
            assert!(
                output.contains(expected),
                "{lang:?} generated client must expose {expected:?} for batch unindex"
            );
        }
    }
}
