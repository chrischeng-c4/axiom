//! Black-box contract for `POST /collections/{id}/docs:truncate` (#3992).
//!
//! A truncate clears indexed documents but keeps the live collection and its
//! schema. It is a write operation. It must reject while the node is degraded
//! or a reshard write fence is armed.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::aof::{replay_aof_into, AofWriter};
use lumen::api::{router, AppState};
use lumen::auth::{AuthConfig, LumenVerifier, COLLECTIONS_RESOURCE};
use lumen::log_entry::RaftLogEntry;
use lumen::storage::Engine;
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};
use lumen::wal::WalRecord;
use service_auth::k8s::{
    AccessReviewOutcome, ResourceAttributes, ReviewBackend, ReviewError, ReviewedIdentity,
    TokenReviewOutcome,
};

const COLLECTION: &str = "notes";
const NAMESPACE: &str = "serving";
const ADMIN: &str = "system:serviceaccount:serving:lumen-admin";
const WRITER: &str = "system:serviceaccount:serving:lumen-writer";
const READER: &str = "system:serviceaccount:serving:lumen-reader";

fn open_server() -> (TestServer, Arc<Engine>) {
    let engine = Arc::new(Engine::new());
    let server = TestServer::new(router(AppState::open(engine.clone()))).expect("open server");
    (server, engine)
}

fn schema(field_count: usize) -> Value {
    let mut fields = serde_json::Map::new();
    for number in 0..field_count {
        fields.insert(format!("kw{number}"), json!({ "type": "keyword" }));
    }
    json!({ "fields": fields })
}

async fn put_schema(server: &TestServer, field_count: usize) -> Value {
    let response = server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&schema(field_count))
        .await;
    response.assert_status_ok();
    response.json()
}

async fn indexed_total(server: &TestServer, field: &str) -> u64 {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "exists": { "field": field } },
            "limit": 100
        }))
        .await;
    response.assert_status_ok();
    response.json::<Value>()["total"]
        .as_u64()
        .expect("search total is a number")
}

async fn index_note(server: &TestServer, external_id: &str, value: &str) {
    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({
            "items": [{ "external_id": external_id, "field": "kw0", "value": value }]
        }))
        .await
        .assert_status_ok();
}

/// Delegated authorization oracle. The only allowed operation for each token
/// is the Kubernetes resource verb mapped from its Lumen role.
struct AuthzOracle {
    tokens: HashMap<&'static str, &'static str>,
    grants: HashSet<(String, String, Option<String>, String)>,
}

impl AuthzOracle {
    fn new() -> Self {
        let tokens = HashMap::from([("admin", ADMIN), ("writer", WRITER), ("reader", READER)]);
        let grants = HashSet::from([
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
        ]);
        Self { tokens, grants }
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

fn durable_schema() -> CreateCollectionRequest {
    CreateCollectionRequest {
        fields: BTreeMap::from([(
            "kw".to_string(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        )]),
    }
}

fn durable_index(external_id: &str, value: &str) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: "durable".into(),
        req: IndexRequest {
            items: vec![IndexItem {
                external_id: external_id.into(),
                field: "kw".into(),
                value: FieldValue::String(value.into()),
                version: None,
            }],
            request_id: None,
        },
    }
}

#[test]
fn truncate_survives_aof_checkpoint_reopen_and_snapshot_restore() {
    let temp = tempfile::tempdir().unwrap();
    let aof_path = temp.path().join("truncate.aof");
    let before_truncate_checkpoint = temp.path().join("before-truncate");
    let after_truncate_checkpoint = temp.path().join("after-truncate");
    let live = Arc::new(Engine::new());
    let mut aof = AofWriter::open(&aof_path).unwrap();

    let entries = [
        RaftLogEntry::CreateCollection {
            collection_id: "durable".into(),
            req: durable_schema(),
        },
        durable_index("old", "old-value"),
    ];
    for (seq, entry) in entries.into_iter().enumerate() {
        live.apply_raft_entry(entry.clone()).unwrap();
        aof.append((seq + 1) as u64, &WalRecord::new(entry))
            .unwrap();
    }
    live.flush_to_segments(&before_truncate_checkpoint, 2)
        .unwrap();
    aof.truncate_through(2).unwrap();

    let truncate = RaftLogEntry::TruncateDocs {
        collection_id: "durable".into(),
    };
    live.apply_raft_entry(truncate.clone()).unwrap();
    aof.append(3, &WalRecord::new(truncate)).unwrap();
    aof.sync().unwrap();
    assert_eq!(live.stats("durable").unwrap().documents_indexed, 0);

    let replayed = Arc::new(Engine::new());
    assert_eq!(
        replayed
            .reopen_from_segment_dir(&before_truncate_checkpoint)
            .unwrap(),
        2
    );
    assert_eq!(replay_aof_into(&replayed, &aof_path, 2).unwrap(), 3);
    assert_eq!(replayed.stats("durable").unwrap().documents_indexed, 0);

    // A checkpoint taken after truncate must also be self-contained: no AOF
    // tail is needed to keep old sealed postings out after cold reopen.
    live.flush_to_segments(&after_truncate_checkpoint, 3)
        .unwrap();
    let reopened = Engine::new();
    assert_eq!(
        reopened
            .reopen_from_segment_dir(&after_truncate_checkpoint)
            .unwrap(),
        3
    );
    assert_eq!(reopened.stats("durable").unwrap().documents_indexed, 0);

    let snapshot = live.snapshot().unwrap();
    let restored = Engine::new();
    restored.restore(snapshot).unwrap();
    assert_eq!(restored.stats("durable").unwrap().documents_indexed, 0);
    restored
        .index(
            "durable",
            IndexRequest {
                items: vec![IndexItem {
                    external_id: "new".into(),
                    field: "kw".into(),
                    value: FieldValue::String("new-value".into()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .unwrap();
    assert_eq!(restored.stats("durable").unwrap().documents_indexed, 1);
}

#[test]
fn truncate_openapi_contract_has_no_request_body_and_a_distinct_operation_id() {
    let spec: Value = serde_json::from_str(&lumen::spec::openapi_json()).unwrap();
    let operation = &spec["paths"]["/collections/{collection_id}/docs:truncate"]["post"];
    assert_eq!(operation["operationId"], "truncate_docs");
    assert!(
        operation.get("requestBody").is_none(),
        "truncate must not accept a JSON body or process-local request_id: {operation}"
    );
    assert!(operation["responses"].get("204").is_some());
    assert!(
        spec["paths"]
            .get("/collections/{collection_id}/docs:merge")
            .is_none(),
        "#3992 must not add the rejected docs:merge surface"
    );
}

#[tokio::test]
async fn truncate_keeps_schema_and_version_clears_search_and_allows_reindex() {
    let (server, _) = open_server();

    assert_eq!(put_schema(&server, 1).await["version"], 1);
    assert_eq!(put_schema(&server, 2).await["version"], 2);
    let version_before_truncate = put_schema(&server, 3).await["version"]
        .as_u64()
        .expect("collection version is a number");
    assert_eq!(version_before_truncate, 3);

    index_note(&server, "old-1", "old-a").await;
    index_note(&server, "old-2", "old-b").await;
    assert_eq!(indexed_total(&server, "kw0").await, 2);

    server
        .post(&format!("/collections/{COLLECTION}/docs:truncate"))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let stats = server
        .get(&format!("/collections/{COLLECTION}/stats"))
        .await;
    stats.assert_status_ok();
    let stats: Value = stats.json();
    assert_eq!(
        stats["documents_indexed"], 0,
        "truncate must clear docs: {stats}"
    );
    for field in ["kw0", "kw1", "kw2"] {
        assert!(
            stats["fields"][field].is_object(),
            "truncate must keep declared field {field}: {stats}"
        );
    }
    assert_eq!(indexed_total(&server, "kw0").await, 0);

    // Extending the pre-existing schema after truncate must continue from the
    // old version. A fresh collection would report version 1 here instead.
    let extended = put_schema(&server, 4).await;
    assert_eq!(
        extended["version"],
        version_before_truncate + 1,
        "truncate must preserve collection schema version: {extended}"
    );
    assert_eq!(extended["fields_count"], 4);

    index_note(&server, "new-1", "new-value").await;
    assert_eq!(indexed_total(&server, "kw0").await, 1);
}

#[tokio::test]
async fn truncate_requires_write_role() {
    let server = delegated_server();
    server
        .put(&format!("/collections/{COLLECTION}"))
        .add_header("authorization", "Bearer admin")
        .json(&schema(1))
        .await
        .assert_status_ok();
    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .add_header("authorization", "Bearer writer")
        .json(&json!({
            "items": [{ "external_id": "row-1", "field": "kw0", "value": "live" }]
        }))
        .await
        .assert_status_ok();

    for token in ["admin", "reader"] {
        server
            .post(&format!("/collections/{COLLECTION}/docs:truncate"))
            .add_header("authorization", format!("Bearer {token}"))
            .await
            .assert_status(StatusCode::FORBIDDEN);
    }

    server
        .post(&format!("/collections/{COLLECTION}/docs:truncate"))
        .add_header("authorization", "Bearer writer")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn truncate_rejects_an_armed_reshard_fence_without_clearing_documents() {
    let (server, _) = open_server();
    put_schema(&server, 1).await;
    index_note(&server, "old-1", "old-value").await;

    server
        .post("/admin/reshard:fence")
        .json(&json!({
            "virtual_bucket_count": 4,
            "buckets": [0],
            "ttl_secs": 30
        }))
        .await
        .assert_status_ok();

    let rejected = server
        .post(&format!("/collections/{COLLECTION}/docs:truncate"))
        .await;
    rejected.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        rejected.json::<Value>()["error"],
        "bucket_write_paused",
        "truncate must not cross an active reshard fence"
    );
    assert_eq!(
        indexed_total(&server, "kw0").await,
        1,
        "a fenced truncate must leave documents unchanged"
    );
}

#[tokio::test]
async fn truncate_returns_storage_full_without_clearing_documents() {
    let (server, engine) = open_server();
    put_schema(&server, 1).await;
    index_note(&server, "old-1", "old-value").await;
    engine.metrics().mark_storage_degraded();

    let rejected = server
        .post(&format!("/collections/{COLLECTION}/docs:truncate"))
        .await;
    rejected.assert_status(StatusCode::INSUFFICIENT_STORAGE);
    assert_eq!(rejected.json::<Value>()["error"], "storage_full");
    assert_eq!(
        indexed_total(&server, "kw0").await,
        1,
        "a storage-full truncate must not clear documents"
    );
}

#[tokio::test]
async fn truncate_rejects_a_body_and_does_not_accept_request_id() {
    let (server, _) = open_server();
    put_schema(&server, 1).await;
    index_note(&server, "old-1", "old-value").await;

    let rejected = server
        .post(&format!("/collections/{COLLECTION}/docs:truncate"))
        .json(&json!({ "request_id": "must-not-be-accepted" }))
        .await;
    rejected.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(indexed_total(&server, "kw0").await, 1);
}
