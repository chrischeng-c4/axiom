// HANDWRITE-BEGIN gap="sift-error-report-store-tests" tracker="1666" reason="Verify fingerprint boundaries, occurrences, correlations, and rebuild equality."
use std::{collections::BTreeMap, sync::Arc};

use chrono::{Duration, Utc};
use sift::{
    projection::{
        error_fingerprint, ErrorLifecycleState, ErrorLifecycleV1, ErrorQuery,
        ErrorReportProjection, Projection, ProjectionRuntime, PROJECTION_ERROR_REPORT_STORE,
    },
    DurableJournal, EventEnvelope, SignalKind, StoredEvent,
};

fn exception(
    cursor: u64,
    id: &str,
    exception_type: &str,
    message: &str,
    stacktrace: &str,
) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        id,
        SignalKind::Exception,
        serde_json::json!({
            "exception.type": exception_type,
            "exception.message": message,
            "exception.stacktrace": stacktrace,
        }),
    );
    event.occurred_at = format!("2026-07-14T00:00:{cursor:02}Z");
    event.observed_at.clone_from(&event.occurred_at);
    event.trace_id = Some(format!("trace-{cursor}"));
    event.span_id = Some(format!("span-{cursor}"));
    event.request_id = Some("request-a".into());
    event.session_id = Some("session-a".into());
    event.resource = BTreeMap::from([("service.name".into(), "checkout".into())]);
    StoredEvent {
        cursor,
        acknowledged_at: event.observed_at.clone(),
        event,
    }
}

#[test]
fn fingerprint_normalizes_volatile_values_but_preserves_boundaries() {
    let left = error_fingerprint(
        "DatabaseError",
        "order 9981 failed for 5ccf3a7e-5138-4a13-aec8-177bf5763a8c",
        "at checkout::load (src/checkout.rs:41)\nat tokio::runtime",
    );
    let right = error_fingerprint(
        "DatabaseError",
        "order 12002 failed for f84243f8-e3c8-4f37-a331-c521d8950f13",
        "at checkout::load (src/checkout.rs:99)\nat tokio::runtime",
    );
    assert_eq!(left, right);
    assert_ne!(
        left,
        error_fingerprint(
            "ValidationError",
            "order 12002 failed for f84243f8-e3c8-4f37-a331-c521d8950f13",
            "at checkout::load (src/checkout.rs:99)"
        )
    );
    assert_ne!(
        right,
        error_fingerprint(
            "DatabaseError",
            "order 12002 failed for f84243f8-e3c8-4f37-a331-c521d8950f13",
            "at checkout::save (src/checkout.rs:99)"
        )
    );
}

#[test]
fn occurrences_are_ordered_correlated_and_lifecycle_is_deterministic() {
    let projection = ErrorReportProjection::new();
    for row in [
        exception(
            2,
            "exception-2",
            "DatabaseError",
            "order 200 failed",
            "at checkout::load (src/checkout.rs:22)",
        ),
        exception(
            1,
            "exception-1",
            "DatabaseError",
            "order 100 failed",
            "at checkout::load (src/checkout.rs:11)",
        ),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let page = projection
        .query(&ErrorQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(page.groups.len(), 1);
    let group = &page.groups[0];
    assert_eq!(group.occurrence_count, 2);
    assert_eq!(
        group
            .occurrences
            .iter()
            .map(|occurrence| occurrence.cursor)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(group.correlations["request_ids"], ["request-a"]);
    assert_eq!(group.correlations["session_ids"], ["session-a"]);
    assert_eq!(group.correlations["trace_ids"], ["trace-1", "trace-2"]);

    let resolved = ErrorLifecycleV1 {
        project: "project-a".into(),
        fingerprint: group.fingerprint.clone(),
        state: ErrorLifecycleState::Resolved,
        muted_until: None,
        actor: "operator".into(),
        reason: Some("fixed".into()),
        occurrence_cursor: 1,
        updated_at: "2026-07-14T00:00:01Z".into(),
        commit_index: 10,
    };
    let reopened = group.clone().apply_lifecycle(Some(&resolved), Utc::now());
    assert_eq!(reopened.state, ErrorLifecycleState::Open);
    assert!(reopened.reopened);

    let expired_mute = ErrorLifecycleV1 {
        state: ErrorLifecycleState::Muted,
        muted_until: Some((Utc::now() - Duration::seconds(1)).to_rfc3339()),
        occurrence_cursor: group.last_cursor,
        ..resolved
    };
    let expired = group
        .clone()
        .apply_lifecycle(Some(&expired_mute), Utc::now());
    assert_eq!(expired.state, ErrorLifecycleState::Open);
    assert_eq!(expired.muted_until, None);
}

#[test]
fn error_projection_rebuilds_equal_from_raw_exceptions() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    for row in [
        exception(1, "exception-1", "Timeout", "after 10 ms", "at api::get:10"),
        exception(2, "exception-2", "Timeout", "after 20 ms", "at api::get:20"),
    ] {
        journal.append(row.event).unwrap();
    }
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_ERROR_REPORT_STORE).unwrap();
    let comparison = runtime
        .rebuild_and_compare(PROJECTION_ERROR_REPORT_STORE)
        .unwrap();
    assert!(comparison.equal);
    assert_eq!(comparison.source_cursor, 2);
}
// HANDWRITE-END
