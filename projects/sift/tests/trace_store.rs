// HANDWRITE-BEGIN gap="sift-trace-store-tests" tracker="1665" reason="Verify topology, missing parents, cycles, links/events, critical path, correlations, ordering, and rebuild equality."
use std::{collections::BTreeMap, sync::Arc};

use sift::{
    projection::{Projection, ProjectionRuntime, TraceProjection, PROJECTION_TRACE_STORE},
    AttributeValue, DurableJournal, EventEnvelope, SignalKind, StoredEvent,
};

fn span(
    cursor: u64,
    trace_id: &str,
    span_id: &str,
    parent_span_id: Option<&str>,
    start: u64,
    end: u64,
) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        format!("{trace_id}-{span_id}"),
        SignalKind::Span,
        serde_json::json!({
            "name": span_id,
            "kind": "server",
            "parent_span_id": parent_span_id,
            "start_time_unix_nano": start,
            "end_time_unix_nano": end,
            "status": {"code": "ok", "message": "complete"},
            "links": [{
                "trace_id": "linked-trace",
                "span_id": "linked-span",
                "attributes": {"link.type": {"type": "string", "value": "follows_from"}}
            }],
            "events": [{
                "name": "db.retry",
                "time_unix_nano": start + 1,
                "attributes": {"retry": {"type": "int", "value": 1}}
            }]
        }),
    );
    event.trace_id = Some(trace_id.into());
    event.span_id = Some(span_id.into());
    event.request_id = Some("request-a".into());
    event.session_id = Some("session-a".into());
    event.attributes.insert(
        "component".into(),
        AttributeValue::String("checkout".into()),
    );
    event.resource = BTreeMap::from([("service.name".into(), "checkout".into())]);
    StoredEvent {
        cursor,
        acknowledged_at: "2026-07-14T00:00:00Z".into(),
        event,
    }
}

#[test]
fn out_of_order_spans_preserve_otel_details_and_build_critical_path() {
    let projection = TraceProjection::new();
    for row in [
        span(1, "trace-a", "leaf", Some("child"), 200, 350),
        span(2, "trace-a", "child", Some("root"), 150, 400),
        span(3, "trace-a", "root", None, 100, 500),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let trace = projection
        .get_trace("project-a", "trace-a")
        .unwrap()
        .unwrap();
    assert_eq!(trace.root_span_ids, ["root"]);
    assert!(!trace.partial);
    assert!(trace.gaps.is_empty());
    assert_eq!(trace.duration_unix_nano, 400);
    assert_eq!(trace.critical_path_span_ids, ["root", "child", "leaf"]);
    assert_eq!(
        trace
            .spans
            .iter()
            .map(|span| span.span_id.as_str())
            .collect::<Vec<_>>(),
        ["root", "child", "leaf"]
    );
    let child = trace
        .spans
        .iter()
        .find(|span| span.span_id == "child")
        .unwrap();
    assert_eq!(child.status_code.as_deref(), Some("ok"));
    assert_eq!(child.links[0].trace_id, "linked-trace");
    assert_eq!(child.events[0].name, "db.retry");
    assert_eq!(child.resource["service.name"], "checkout");
    assert_eq!(trace.correlation_ids["request_ids"], ["request-a"]);
    assert_eq!(trace.correlation_ids["session_ids"], ["session-a"]);
    assert_eq!(trace.correlation_ids["linked_trace_ids"], ["linked-trace"]);
}

#[test]
fn missing_parents_and_cycles_are_explicit_partial_trace_diagnostics() {
    let projection = TraceProjection::new();
    Projection::apply_idempotent(
        &projection,
        &span(1, "missing-trace", "child", Some("absent"), 10, 20),
    )
    .unwrap();
    let missing = projection
        .get_trace("project-a", "missing-trace")
        .unwrap()
        .unwrap();
    assert!(missing.partial);
    assert_eq!(missing.root_span_ids, ["child"]);
    assert!(missing
        .gaps
        .iter()
        .any(|gap| gap == "missing_parent:child:absent"));

    Projection::apply_idempotent(&projection, &span(2, "cycle-trace", "a", Some("b"), 10, 30))
        .unwrap();
    Projection::apply_idempotent(&projection, &span(3, "cycle-trace", "b", Some("a"), 15, 25))
        .unwrap();
    let cycle = projection
        .get_trace("project-a", "cycle-trace")
        .unwrap()
        .unwrap();
    assert!(cycle.partial);
    assert!(!cycle.cycles.is_empty());
    assert!(cycle.critical_path_span_ids.len() <= cycle.spans.len());
}

#[test]
fn trace_projection_rebuilds_equal_from_raw_spans() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    for row in [
        span(1, "trace-a", "child", Some("root"), 20, 40),
        span(2, "trace-a", "root", None, 10, 50),
    ] {
        journal.append(row.event).unwrap();
    }
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_TRACE_STORE).unwrap();
    let comparison = runtime.rebuild_and_compare(PROJECTION_TRACE_STORE).unwrap();
    assert!(comparison.equal);
    assert_eq!(comparison.source_cursor, 2);
}

// HANDWRITE-END
