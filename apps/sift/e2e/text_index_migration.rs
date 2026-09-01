use sift::{
    projection::{LogQuery, LoggingProjection, Projection},
    EventEnvelope, SignalKind, StoredEvent,
};

fn stored_log(cursor: u64, id: &str, body: &str) -> StoredEvent {
    StoredEvent {
        cursor,
        acknowledged_at: "2026-08-31T00:00:00Z".to_string(),
        event: EventEnvelope::for_project(
            "project-a",
            "prod",
            id,
            SignalKind::Log,
            serde_json::json!({"message": body}),
        ),
    }
}

#[test]
fn logging_search_and_snapshot_use_the_shared_rebuildable_index() {
    let projection = LoggingProjection::with_max_records(10).unwrap();
    projection
        .apply_idempotent(&stored_log(1, "log-1", "disk error on shard seven"))
        .unwrap();
    projection
        .apply_idempotent(&stored_log(2, "log-2", "request completed"))
        .unwrap();

    let mut query = LogQuery::for_project("project-a");
    query.text = Some("disk error".to_string());
    assert_eq!(projection.query(&query).unwrap().records.len(), 1);

    let restored = LoggingProjection::with_max_records(10).unwrap();
    restored.restore(&projection.snapshot().unwrap()).unwrap();
    assert_eq!(restored.query(&query).unwrap().records[0].event_id, "log-1");
}

#[test]
fn sift_has_no_production_dependency_on_an_app_index() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read Sift manifest");
    let production = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("production manifest section");
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/projection/logging.rs"
    ))
    .expect("read logging projection source");

    assert!(production.contains("index-text ="));
    assert!(!production.contains("lumen ="));
    assert!(source.contains("index_text::{"));
    assert!(source.contains("MemoryTextIndex"));
    assert!(!source.contains("lumen::"));
}
