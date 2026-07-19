// HANDWRITE-BEGIN gap="sift-audit-change-store-tests" tracker="1668" reason="Verify normalization, immutability, chain integrity, correlation, and rebuild equality."
use std::{collections::BTreeMap, sync::Arc};

use chrono::{TimeZone, Utc};
use sift::{
    projection::{
        AuditChangeProjection, AuditQuery, Projection, ProjectionRuntime,
        PROJECTION_AUDIT_CHANGE_STORE,
    },
    AttributeValue, DurableJournal, EventEnvelope, SignalKind, StoredEvent,
};

fn audit(
    cursor: u64,
    id: &str,
    project: &str,
    signal: SignalKind,
    actor: &str,
    action: &str,
) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        project,
        "prod",
        id,
        signal,
        serde_json::json!({
            "actor": actor,
            "subject": "checkout-api",
            "action": action,
            "target": "deployment/checkout",
            "version": "v2"
        }),
    );
    event.occurred_at = format!("2026-07-14T00:00:{cursor:02}Z");
    event.observed_at.clone_from(&event.occurred_at);
    event.resource = BTreeMap::from([("service.name".into(), "checkout".into())]);
    event.attributes.insert(
        "deployment.environment".into(),
        AttributeValue::String("prod".into()),
    );
    event.trace_id = Some("trace-deploy".into());
    event.span_id = Some("span-deploy".into());
    event.request_id = Some("request-deploy".into());
    event.session_id = Some("session-deploy".into());
    StoredEvent {
        cursor,
        acknowledged_at: event.observed_at.clone(),
        event,
    }
}

#[test]
fn records_are_normalized_immutable_correlated_and_hash_chained_per_project() {
    let projection = AuditChangeProjection::new();
    let first = audit(
        1,
        "audit-1",
        "project-a",
        SignalKind::AuditEvent,
        "deployer",
        "deployment.authorize",
    );
    let second = audit(
        2,
        "change-1",
        "project-a",
        SignalKind::ChangeEvent,
        "deployer",
        "deployment.apply",
    );
    let other = audit(
        3,
        "audit-b",
        "project-b",
        SignalKind::AuditEvent,
        "other",
        "config.read",
    );
    for row in [&first, &second, &other] {
        Projection::apply_idempotent(&projection, row).unwrap();
    }
    let mut replacement = second.clone();
    replacement.cursor = 4;
    replacement.event.payload["actor"] = serde_json::json!("attacker");
    Projection::apply_idempotent(&projection, &replacement).unwrap();

    let now = Utc.with_ymd_and_hms(2026, 7, 14, 1, 0, 0).unwrap();
    let page = projection
        .query(&AuditQuery::for_project("project-a"), &[], now)
        .unwrap();
    assert_eq!(page.records.len(), 2);
    assert!(page.chain_valid);
    assert_eq!(page.records[0].previous_hash, "GENESIS");
    assert_eq!(page.records[1].previous_hash, page.records[0].record_hash);
    assert_eq!(page.records[1].actor, "deployer");
    assert_eq!(page.records[1].subject.as_deref(), Some("checkout-api"));
    assert_eq!(
        page.records[1].target.as_deref(),
        Some("deployment/checkout")
    );
    assert_eq!(page.records[1].trace_id.as_deref(), Some("trace-deploy"));
    assert_eq!(
        page.records[1].request_id.as_deref(),
        Some("request-deploy")
    );
    assert_eq!(page.records[1].resource["service.name"], "checkout");
    projection.verify_integrity().unwrap();

    let other_page = projection
        .query(&AuditQuery::for_project("project-b"), &[], now)
        .unwrap();
    assert_eq!(other_page.records[0].previous_hash, "GENESIS");
}

#[test]
fn snapshot_restore_rejects_record_tampering() {
    let projection = AuditChangeProjection::new();
    Projection::apply_idempotent(
        &projection,
        &audit(
            1,
            "audit-1",
            "project-a",
            SignalKind::AuditEvent,
            "admin",
            "config.update",
        ),
    )
    .unwrap();
    let snapshot = Projection::snapshot(&projection).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    value["records"]["1"]["payload"]["actor"] = serde_json::json!("tampered");
    let tampered = serde_json::to_vec(&value).unwrap();
    let restored = AuditChangeProjection::new();
    assert!(Projection::restore(&restored, &tampered).is_err());
}

#[test]
fn audit_change_projection_rebuilds_equal_from_raw_events() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    for row in [
        audit(
            1,
            "audit-1",
            "project-a",
            SignalKind::AuditEvent,
            "admin",
            "config.authorize",
        ),
        audit(
            2,
            "change-1",
            "project-a",
            SignalKind::ChangeEvent,
            "admin",
            "config.update",
        ),
    ] {
        journal.append(row.event).unwrap();
    }
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_AUDIT_CHANGE_STORE).unwrap();
    let comparison = runtime
        .rebuild_and_compare(PROJECTION_AUDIT_CHANGE_STORE)
        .unwrap();
    assert!(comparison.equal);
    assert_eq!(comparison.source_cursor, 2);
}
// HANDWRITE-END
