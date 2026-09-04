// HANDWRITE-BEGIN gap="sift-profile-blob-tests" tracker="1669" reason="Verify blob-before-ack, bounded raw bytes, missing/corrupt rejection, and OTLP API authorization."
use std::{collections::HashMap, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use prost::Message;
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    projection::{ProjectionRuntime, PROJECTION_PROFILE_STORE},
    protected_router,
    storage::BlobStore,
    ContentBlobRef, DurableJournal, EventEnvelope, ServiceState, SignalKind,
};
use tower::ServiceExt;

#[derive(Clone, PartialEq, Message)]
struct ProfilesRequest {
    #[prost(message, repeated, tag = "1")]
    resource_profiles: Vec<TestResourceProfiles>,
    #[prost(message, optional, tag = "2")]
    dictionary: Option<TestDictionary>,
}

#[derive(Clone, PartialEq, Message)]
struct TestResourceProfiles {
    #[prost(message, repeated, tag = "2")]
    scope_profiles: Vec<TestScopeProfiles>,
}

#[derive(Clone, PartialEq, Message)]
struct TestScopeProfiles {
    #[prost(message, repeated, tag = "2")]
    profiles: Vec<TestProfile>,
}

#[derive(Clone, PartialEq, Message)]
struct TestProfile {
    #[prost(fixed64, tag = "3")]
    time_unix_nano: u64,
    #[prost(uint64, tag = "4")]
    duration_nano: u64,
    #[prost(bytes, tag = "7")]
    profile_id: Vec<u8>,
    #[prost(string, tag = "9")]
    original_payload_format: String,
    #[prost(bytes, tag = "10")]
    original_payload: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct TestDictionary {}

fn large_profile_event(id: &str) -> EventEnvelope {
    let samples = (0..3_000)
        .map(|index| {
            serde_json::json!({
                "frames":["root","worker"],
                "values":[index % 7 + 1],
                "attributeIndices":[]
            })
        })
        .collect::<Vec<_>>();
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        id,
        SignalKind::Profile,
        serde_json::json!({
            "profile": {
                "profileId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "timeUnixNano":"1783987200000000000",
                "durationNano":"1000000",
                "samples":samples
            },
            "dictionary": {}
        }),
    );
    event.occurred_at = "2026-07-14T00:00:00Z".into();
    event.observed_at.clone_from(&event.occurred_at);
    event
        .resource
        .insert("service.name".into(), "checkout".into());
    event
}

fn verifier() -> Arc<SiftVerifier> {
    Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([
            (
                "writer-token".into(),
                TokenClaims {
                    subject: "profiler".into(),
                    roles: HashMap::from([("project-a".into(), Role::Write)]),
                },
            ),
            (
                "reader-token".into(),
                TokenClaims {
                    subject: "sre".into(),
                    roles: HashMap::from([("project-a".into(), Role::Read)]),
                },
            ),
            (
                "other-token".into(),
                TokenClaims {
                    subject: "other".into(),
                    roles: HashMap::from([("project-b".into(), Role::Read)]),
                },
            ),
        ]),
    }))
}

/// A profile timestamp inside the default 30-day profile retention window, so
/// the query below keeps returning the record regardless of the calendar date
/// (a fixed 2026-07-14 stamp expired from the projection on 2026-08-13).
fn recent_profile_time_unix_nano() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after the Unix epoch");
    (now - std::time::Duration::from_secs(60)).as_nanos() as u64
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn large_profile_blob_is_durable_before_bounded_raw_acknowledgement() {
    let temp = tempfile::tempdir().unwrap();
    let journal = DurableJournal::open(temp.path()).unwrap();
    let result = journal
        .append(large_profile_event("large-profile"))
        .unwrap();
    assert_eq!(result.raw_cursor, 1);
    let stored = journal.query(Default::default()).unwrap().remove(0);
    assert!(stored.event.payload.get("profileBlob").is_some());
    assert_eq!(stored.event.blob_refs.len(), 1);
    assert!(serde_json::to_vec(&stored.event).unwrap().len() < 4_096);
    let bytes = journal
        .storage()
        .read_blob(&stored.event.blob_refs[0].hash)
        .unwrap();
    assert_eq!(bytes.len() as u64, stored.event.blob_refs[0].size);
    let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        payload["profile"]["samples"].as_array().unwrap().len(),
        3_000
    );
}

#[test]
fn missing_referenced_blob_is_rejected_and_deleted_blob_blocks_projection() {
    let missing_temp = tempfile::tempdir().unwrap();
    let missing_journal = DurableJournal::open(missing_temp.path()).unwrap();
    let reference = ContentBlobRef {
        hash: format!("sha256:{}", "0".repeat(64)),
        size: 10,
        encoding: "application/json".into(),
    };
    let mut missing = EventEnvelope::for_project(
        "project-a",
        "prod",
        "missing-profile",
        SignalKind::Profile,
        serde_json::json!({"profileBlob": reference}),
    );
    missing
        .resource
        .insert("service.name".into(), "checkout".into());
    missing.blob_refs.push(reference);
    assert!(missing_journal.append(missing).is_err());
    assert!(missing_journal
        .query(Default::default())
        .unwrap()
        .is_empty());

    let deleted_temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(deleted_temp.path()).unwrap());
    journal
        .append(large_profile_event("deleted-profile"))
        .unwrap();
    let stored = journal.query(Default::default()).unwrap().remove(0);
    let blob_store = BlobStore::open(deleted_temp.path(), 65_536).unwrap();
    std::fs::remove_file(
        blob_store
            .path_for_hash(&stored.event.blob_refs[0].hash)
            .unwrap(),
    )
    .unwrap();
    let runtime = ProjectionRuntime::open(deleted_temp.path(), journal).unwrap();
    assert!(runtime.catch_up(PROJECTION_PROFILE_STORE).is_err());
}

#[tokio::test]
async fn otlp_protobuf_profile_payload_and_query_are_project_scoped() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = protected_router(state.clone(), verifier());
    let source_payload = vec![b'p'; 70_000];
    let request = ProfilesRequest {
        resource_profiles: vec![TestResourceProfiles {
            scope_profiles: vec![TestScopeProfiles {
                profiles: vec![TestProfile {
                    time_unix_nano: recent_profile_time_unix_nano(),
                    duration_nano: 1_000_000,
                    profile_id: vec![0x44; 16],
                    original_payload_format: "pprof".into(),
                    original_payload: source_payload.clone(),
                }],
            }],
        }],
        dictionary: Some(TestDictionary {}),
    }
    .encode_to_vec();

    let ingested = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/profiles")
                .header("authorization", "Bearer writer-token")
                .header("content-type", "application/x-protobuf")
                .header("x-sift-project", "project-a")
                .body(Body::from(request))
                .unwrap(),
        )
        .await
        .unwrap();
    if ingested.status() != StatusCode::OK {
        let status = ingested.status();
        let body = to_bytes(ingested.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        panic!(
            "profile protobuf ingest returned {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }
    let stored = state.journal().query(Default::default()).unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].event.blob_refs.len(), 1);
    assert_eq!(
        state
            .journal()
            .storage()
            .read_blob(&stored[0].event.blob_refs[0].hash)
            .unwrap(),
        source_payload
    );
    assert!(serde_json::to_vec(&stored[0].event).unwrap().len() < 8_192);

    let forbidden = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/profiles:query")
                .header("authorization", "Bearer other-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project":"project-a"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let queried = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/profiles:query")
                .header("authorization", "Bearer reader-token")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"project":"project-a","min_cursor":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    if queried.status() != StatusCode::OK {
        let status = queried.status();
        let body = to_bytes(queried.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap();
        panic!(
            "profile query returned {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }
    let queried = json_body(queried).await;
    assert_eq!(queried["records"].as_array().unwrap().len(), 1);
    assert_eq!(
        queried["records"][0]["profile_id"],
        "44444444444444444444444444444444"
    );
    assert_eq!(queried["projection_cursor"], 1);
}
// HANDWRITE-END
