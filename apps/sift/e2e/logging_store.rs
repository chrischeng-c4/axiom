// HANDWRITE-BEGIN gap="sift-logging-store-tests" tracker="1664" reason="Verify golden GCP/OTel records, filters, text, retention, coexistence identity, and rebuild equality."
use std::{collections::BTreeMap, sync::Arc};

use sift::{
    projection::{
        LogQuery, LoggingProjection, Projection, ProjectionRuntime, PROJECTION_LOGGING_STORE,
    },
    AttributeValue, DurableJournal, EventEnvelope, SignalKind, StoredEvent,
};

fn stored(cursor: u64, mut event: EventEnvelope) -> StoredEvent {
    event.occurred_at = format!("2026-07-14T00:00:0{cursor}Z");
    event.observed_at = event.occurred_at.clone();
    StoredEvent {
        cursor,
        acknowledged_at: event.observed_at.clone(),
        event,
    }
}

fn gcp_log(cursor: u64, id: &str, message: &str) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "payments",
        id,
        SignalKind::Log,
        serde_json::json!({"jsonPayload": {"message": message, "attempt": cursor}}),
    );
    event.severity = Some("ERROR".into());
    event.trace_id = Some("trace-a".into());
    event.span_id = Some("span-a".into());
    event.request_id = Some("request-a".into());
    event.session_id = Some("session-a".into());
    event.resource = BTreeMap::from([
        ("gcp.resource.type".into(), "k8s_container".into()),
        ("k8s.cluster.name".into(), "prod".into()),
        ("k8s.namespace.name".into(), "payments".into()),
        ("k8s.pod.name".into(), "checkout-1".into()),
        ("k8s.container.name".into(), "app".into()),
        ("service.name".into(), "checkout".into()),
    ]);
    event.attributes.insert(
        "deployment.version".into(),
        AttributeValue::String("v2".into()),
    );
    stored(cursor, event)
}

#[test]
fn gcp_and_otel_golden_records_preserve_structure_correlation_and_search() {
    let projection = LoggingProjection::new().unwrap();
    Projection::apply_idempotent(
        &projection,
        &gcp_log(1, "gcp-insert-1", "pod failed during checkout"),
    )
    .unwrap();

    let mut otel = EventEnvelope::for_project(
        "project-a",
        "payments",
        "otel-2",
        SignalKind::Log,
        serde_json::json!({"body": "checkout recovered", "attempt": 2}),
    );
    otel.severity = Some("INFO".into());
    otel.resource
        .insert("service.name".into(), "checkout".into());
    Projection::apply_idempotent(&projection, &stored(2, otel)).unwrap();

    let ignored = EventEnvelope::for_project(
        "project-a",
        "payments",
        "span-3",
        SignalKind::Span,
        serde_json::json!({"name": "checkout"}),
    );
    Projection::apply_idempotent(&projection, &stored(3, ignored)).unwrap();

    let mut query = LogQuery::for_project("project-a");
    query.text = Some("pod failed".into());
    query.severity = Some("error".into());
    query.resource_type = Some("k8s_container".into());
    query.trace_id = Some("trace-a".into());
    query.request_id = Some("request-a".into());
    query.attribute_equals = BTreeMap::from([(
        "deployment.version".into(),
        AttributeValue::String("v2".into()),
    )]);
    let page = projection.query(&query).unwrap();
    assert_eq!(page.records.len(), 1);
    let record = &page.records[0];
    assert_eq!(record.event_id, "gcp-insert-1");
    assert_eq!(record.json_payload["attempt"], 1);
    assert_eq!(record.resource["k8s.pod.name"], "checkout-1");
    assert_eq!(record.span_id.as_deref(), Some("span-a"));
    assert_eq!(record.session_id.as_deref(), Some("session-a"));
    assert_eq!(record.coexistence_key, "project-a:gcp-insert-1");

    let mut otel_query = LogQuery::for_project("project-a");
    otel_query.text = Some("checkout recovered".into());
    assert_eq!(
        projection.query(&otel_query).unwrap().records[0].event_id,
        "otel-2"
    );
}

#[test]
fn retention_snapshot_and_restore_are_deterministic() {
    let projection = LoggingProjection::with_max_records(2).unwrap();
    for cursor in 1..=3 {
        Projection::apply_idempotent(
            &projection,
            &gcp_log(cursor, &format!("event-{cursor}"), "repeated failure"),
        )
        .unwrap();
    }
    let page = projection
        .query(&LogQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(
        page.records
            .iter()
            .map(|row| row.cursor)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );

    let snapshot = Projection::snapshot(&projection).unwrap();
    let digest = Projection::semantic_digest(&projection).unwrap();
    let restored = LoggingProjection::with_max_records(2).unwrap();
    Projection::restore(&restored, &snapshot).unwrap();
    assert_eq!(Projection::semantic_digest(&restored).unwrap(), digest);
    assert_eq!(
        restored
            .query(&LogQuery::for_project("project-a"))
            .unwrap()
            .records
            .len(),
        2
    );
}

#[test]
fn logging_projection_rebuilds_equal_from_the_raw_journal() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal
        .append(gcp_log(1, "event-1", "pod failed").event)
        .unwrap();
    journal
        .append(gcp_log(2, "event-2", "pod recovered").event)
        .unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();
    let comparison = runtime
        .rebuild_and_compare(PROJECTION_LOGGING_STORE)
        .unwrap();
    assert!(comparison.equal);
    assert_eq!(comparison.source_cursor, 2);
    assert_eq!(comparison.rebuilt_cursor, 2);
}

#[test]
fn identical_event_ids_are_isolated_by_project_in_records_and_text_index() {
    let projection = LoggingProjection::new().unwrap();
    let project_a = gcp_log(1, "shared-id", "alpha-only marker");
    let mut project_b = gcp_log(2, "shared-id", "bravo-only marker");
    project_b.event.project = "project-b".into();

    Projection::apply_idempotent(&projection, &project_a).unwrap();
    Projection::apply_idempotent(&projection, &project_b).unwrap();

    let mut query_a = LogQuery::for_project("project-a");
    query_a.text = Some("alpha-only".into());
    let mut query_b = LogQuery::for_project("project-b");
    query_b.text = Some("bravo-only".into());
    assert_eq!(projection.query(&query_a).unwrap().records.len(), 1);
    assert_eq!(projection.query(&query_b).unwrap().records.len(), 1);

    let snapshot: serde_json::Value =
        serde_json::from_slice(&Projection::snapshot(&projection).unwrap()).unwrap();
    assert_eq!(snapshot["records"].as_object().unwrap().len(), 2);
}

#[test]
fn event_id_reuse_after_the_receipt_window_keeps_both_accepted_rows() {
    let projection = LoggingProjection::new().unwrap();
    Projection::apply_idempotent(
        &projection,
        &gcp_log(1, "reused-id", "first accepted marker"),
    )
    .unwrap();
    Projection::apply_idempotent(
        &projection,
        &gcp_log(2, "reused-id", "second accepted marker"),
    )
    .unwrap();

    let page = projection
        .query(&LogQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(
        page.records
            .iter()
            .map(|record| record.cursor)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    for marker in ["first accepted", "second accepted"] {
        let mut query = LogQuery::for_project("project-a");
        query.text = Some(marker.into());
        assert_eq!(projection.query(&query).unwrap().records.len(), 1);
    }

    let snapshot = Projection::snapshot(&projection).unwrap();
    let restored = LoggingProjection::new().unwrap();
    Projection::restore(&restored, &snapshot).unwrap();
    assert_eq!(
        restored
            .query(&LogQuery::for_project("project-a"))
            .unwrap()
            .records
            .len(),
        2
    );
}

// HANDWRITE-END
