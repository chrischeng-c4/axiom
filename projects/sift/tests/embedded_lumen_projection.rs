// HANDWRITE-BEGIN gap="sift-embedded-lumen-tests" tracker="1660" reason="Verify fixed-field text/keyword/range search, snapshot restore, and absence of a second service boundary."
use std::{collections::BTreeMap, fs};

use sift::{
    projection::{EmbeddedLumenProjection, Projection},
    EventEnvelope, SignalKind, StoredEvent,
};

fn stored(cursor: u64, id: &str, project: &str, message: &str) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        project,
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({
            "message": message,
            "secret": "must-not-become-a-field"
        }),
    );
    event.resource = BTreeMap::from([("service.name".into(), "checkout".into())]);
    StoredEvent {
        cursor,
        acknowledged_at: "2026-07-14T00:00:00Z".into(),
        event,
    }
}

#[test]
fn embedded_lumen_indexes_only_the_fixed_projection_schema() {
    let projection = EmbeddedLumenProjection::new().unwrap();
    Projection::apply_idempotent(
        &projection,
        &stored(1, "evt-1", "alpha", "database timeout while charging card"),
    )
    .unwrap();
    Projection::apply_idempotent(
        &projection,
        &stored(2, "evt-2", "beta", "checkout recovered"),
    )
    .unwrap();

    assert_eq!(
        projection.search_text("database timeout", 10).unwrap(),
        vec!["evt-1"]
    );
    assert_eq!(
        projection.search_keyword("project", "alpha", 10).unwrap(),
        vec!["evt-1"]
    );
    assert_eq!(
        projection
            .search_number_range("cursor", Some(2.0), Some(2.0), 10)
            .unwrap(),
        vec!["evt-2"]
    );
    assert!(projection
        .search_keyword("secret", "must-not-become-a-field", 10)
        .is_err());
}

#[test]
fn embedded_lumen_snapshot_restores_the_same_semantic_view() {
    let projection = EmbeddedLumenProjection::new().unwrap();
    Projection::apply_idempotent(
        &projection,
        &stored(1, "evt-1", "alpha", "database timeout"),
    )
    .unwrap();
    Projection::apply_idempotent(
        &projection,
        &stored(2, "evt-2", "alpha", "database recovered"),
    )
    .unwrap();

    let snapshot = Projection::snapshot(&projection).unwrap();
    let digest = Projection::semantic_digest(&projection).unwrap();
    let restored = EmbeddedLumenProjection::new().unwrap();
    Projection::restore(&restored, &snapshot).unwrap();

    assert_eq!(Projection::semantic_digest(&restored).unwrap(), digest);
    assert_eq!(
        restored.search_text("database", 10).unwrap(),
        vec!["evt-1", "evt-2"]
    );
}

#[test]
fn sift_embeds_lumen_without_a_second_service_or_raft_boundary() {
    let cargo = fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"))).unwrap();
    assert!(cargo.contains("lumen = { path = \"../../apps/lumen\", default-features = false }"));

    let adapter = fs::read_to_string(format!(
        "{}/src/projection/lumen.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    for forbidden in ["lumen::api", "lumen::raft", "lumen::wal"] {
        assert!(!adapter.contains(forbidden), "found forbidden {forbidden}");
    }
}

// HANDWRITE-END
